mod handlers;
mod advertiser;
mod utils;

use embassy_futures::{select::select};
use embassy_time::{Duration};
use esp_alloc as _;
use esp_backtrace as _;
use log::*;
use static_cell::StaticCell;
use trouble_host::prelude::*;
use crate::comm::ble::handlers::{gatt_events::gatt_events_task, notifier::custom_task};
use crate::comm::ble::advertiser::advertise;
use crate::comm::wifi::models::{ENTIRE_SSID_PAGE_SIZE, MAX_PASSWORD_LEN};
use crate::errors::ble_error::BleError;
use crate::prelude::AppState;

/// Max number of connections
pub const BLE_CONNECTIONS_MAX: usize = 1;
/// Max number of L2CAP channels.
pub const BLE_L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

mod ble_gatt_server_uuids {
    use trouble_host::prelude::*;

    pub const SERVICE_UUID: Uuid = uuid!("5f435fa5-adee-4e9a-b9a3-b812d2628906");

    pub const WIFI_SCAN_CMD: Uuid = uuid!("d88a7d46-9313-4240-915a-a2320fa3a6e5");
    pub const WIFI_GET_STATUS: Uuid = uuid!("878b58f2-4d44-4178-ae8f-a9e56d607e9e");

    pub const WIFI_GET_PAGES_COUNT: Uuid = uuid!("0ce70db8-be92-4160-a2d3-588e8b248b95");
    pub const WIFI_SELECT_PAGE: Uuid = uuid!("6839a19d-8a4b-4691-89cc-7a312c1efe54");
    pub const WIFI_GET_PAGE_DATA: Uuid = uuid!("9c0c07d7-0435-4a9d-b999-369c8f646252");

    pub const WIFI_SET_SSID_INDEX: Uuid = uuid!("824f9460-5d76-4498-a549-0020100907bc");
    pub const WIFI_SET_PASSWORD: Uuid = uuid!("273d7528-c072-4fe6-b29b-c1e468f039f2");
    pub const WIFI_CONNECT: Uuid = uuid!("2c1f2d97-5c53-435b-940c-c36cf349ca53");

    pub const WIFI_DISCONNECT: Uuid = uuid!("61cd3e5f-0a78-4318-9891-f1ef74a522e3");

    pub const WIFI_LOCAL_TEST: Uuid = uuid!("54477984-44ea-4dbb-8740-b597f3532d9b");

    pub const STATUS_CODE: Uuid = uuid!("7df744c9-3a9b-4df6-80f3-ec8c3b77338e");
}

/// GATT Server definition
#[gatt_server]
pub struct BleGATTServer {
    general_service: GeneralService,
}

/// Server service
#[gatt_service(uuid = ble_gatt_server_uuids::SERVICE_UUID)]
pub struct GeneralService {
    // General Wi-Fi commands
    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_SCAN_CMD, write)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_scan_cmd")]
    wifi_scan_cmd: bool,

    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_GET_STATUS, read, notify)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_get_status")]
    wifi_get_status: u8, // 0 - Idle, 1 - Scanning, 2 - Connected

    // Wi-Fi scanning
    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_GET_PAGES_COUNT, read, notify)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_get_pages_count")]
    wifi_get_pages_count: u8,

    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_SELECT_PAGE, write)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_select_page")]
    wifi_select_page: u8,

    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_GET_PAGE_DATA, read, value = [0u8; ENTIRE_SSID_PAGE_SIZE] )]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_get_page_data")]
    wifi_get_page_data: [u8; ENTIRE_SSID_PAGE_SIZE],


    // Wi-Fi connection
    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_SET_SSID_INDEX, write)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_set_ssid_index")]
    wifi_set_ssid_index: u8,

    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_SET_PASSWORD, write, value = [0u8; MAX_PASSWORD_LEN] )]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_set_password")]
    wifi_set_password: [u8; MAX_PASSWORD_LEN],

    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_CONNECT, write)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_connect")]
    wifi_connect: bool,

    // Wi-Fi disconnect
    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_DISCONNECT, write)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_disconnect")]
    wifi_disconnect: bool,
    

    // Wi-Fi tests
    #[characteristic(uuid = ble_gatt_server_uuids::WIFI_LOCAL_TEST, write)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "wifi_local_test")]
    wifi_local_test: bool,


    // General fields
    #[characteristic(uuid = ble_gatt_server_uuids::STATUS_CODE, read, notify)]
    #[descriptor(uuid = descriptors::CHARACTERISTIC_USER_DESCRIPTION, read, value = "status_code")]
    status_code: u8,
}


pub type BleResources = HostResources<DefaultPacketPool, BLE_CONNECTIONS_MAX, BLE_L2CAP_CHANNELS_MAX>;
pub type BleController = ExternalController<esp_radio::ble::controller::BleConnector<'static>, 1>;
pub type BleStack = Stack<'static, BleController, DefaultPacketPool>;


pub fn init_gatt_server(
    controller: BleController,
) -> (
    &'static BleGATTServer<'static>,
    Peripheral<'static, BleController, DefaultPacketPool>,
    Runner<'static, BleController, DefaultPacketPool>,
    &'static BleStack
) {

    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0x04]);

    let resources: HostResources<DefaultPacketPool, BLE_CONNECTIONS_MAX, BLE_L2CAP_CHANNELS_MAX> = HostResources::new();
    static RESOURCES_CELL: StaticCell<HostResources<DefaultPacketPool, BLE_CONNECTIONS_MAX, BLE_L2CAP_CHANNELS_MAX>> = StaticCell::new();
    let resources_ref = RESOURCES_CELL.init(resources);

    static STACK_CELL: StaticCell<BleStack> = StaticCell::new();
    let stack = trouble_host::new(controller, resources_ref).set_random_address(address);
    let stack_ref = STACK_CELL.init(stack);

    let Host {peripheral, runner, ..} =
        stack_ref.build();

    static SERVER_CELL: StaticCell<BleGATTServer<'static>> = StaticCell::new();
    let server = BleGATTServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "ESP32 board",
        appearance: &appearance::control_device::GENERIC_CONTROL_DEVICE,
    })).map_err(BleError::ServerInitializationFailed).unwrap();

    let server_ref = SERVER_CELL.init(server);

    (server_ref, peripheral, runner, stack_ref)
}

pub async fn ble_run(
    server: &'static BleGATTServer<'_>,
    peripheral: &mut Peripheral<'static, BleController, DefaultPacketPool>,
    stack: &'static BleStack,
    app_state: AppState
) {
    // Use with_timeout to limit how long we wait for a phone to connect
    let ad_result = embassy_time::with_timeout(
        Duration::from_secs(30),
        advertise("ESP32 board", peripheral, server)
    ).await;

    //TODO: Move to the task, when connection is established
    match ad_result {
        Ok(Ok(conn)) => {
            info!("Connection established! Entering active mode.");
            let a = gatt_events_task(server, &conn, app_state);
            let b = custom_task(server, &conn, stack);
            select(a, b).await;
            info!("Disconnected. Returning to IDLE.");
        }
        Ok(Err(e)) => warn!("Advertising error: {:?}", e),
        Err(_) => {
            // This is the TIMEOUT case
            warn!("No one connected within 30 seconds. Turning Bluetooth OFF.");
        }
    }
}

/// This is a background task that is required to run forever alongside any other BLE tasks.
///
/// ## Alternative
///
/// If you didn't require this to be generic for your application, you could statically spawn this
/// with i.e.
///
/// ```rust,ignore
///
/// #[embassy_executor::task]
/// async fn ble_task(mut runner: Runner<'static, SoftdeviceController<'static>>) {
///     runner.run().await;
/// }
///
/// spawner.must_spawn(ble_task(runner));
/// ```
pub async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            panic!("[ble_task] error: {:?}", e);
        }
    }
}
