use esp_hal::rng::Rng;
use esp_radio::wifi;
use log::info;
use smoltcp::iface::{SocketSet, SocketStorage};
use static_cell::StaticCell;
use crate::comm::wifi::utils::create_interface;
use crate::prelude::AppState;
use crate::utils::timestamp_now;


pub enum WifiRunnerCommand {
    /// Try to connect to Wi-Fi via storage-saved credentials
    Reconnect, // Runtime default
    /// Scanning nearby networks and save to app_state
    Scan,
    /// Send ping to router
    PingLocal
}

#[embassy_executor::task]
pub async fn wifi_runner(
    app_state: AppState,
    wifi_controller: wifi::WifiController<'static>,
    mut wifi_device: wifi::WifiDevice<'static>,
    rng: Rng
) {
    // Buffers
    static RX_BUF: StaticCell<[u8; 1536]> = StaticCell::new();
    static TX_BUF: StaticCell<[u8; 1536]> = StaticCell::new();
    let rx_buffer = RX_BUF.init([0u8; 1536]);
    let tx_buffer = TX_BUF.init([0u8; 1536]);

    // Socket storage
    static SOCKET_ENTRIES: StaticCell<[SocketStorage; 3]> = StaticCell::new();
    let socket_set_entries = SOCKET_ENTRIES.init(Default::default());

    // Init network interface
    let now = timestamp_now() as i64;
    let interface = create_interface(
        &mut wifi_device,
        smoltcp::time::Instant::from_micros(now)
    );

    // Init socket set
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();
    socket_set.add(dhcp_socket);

    // Init stack
    let mut stack = blocking_network_stack::Stack::new(
        interface,
        wifi_device,
        socket_set,
        timestamp_now,
        rng.random()
    );
    info!("Stack initialized");

    // Connect to Wi-Fi network
    loop {
        stack.work();

        let command = app_state.wifi_command.receive().await;

        // match command {
        //
        // }


    }
}