use embassy_time::Timer;
use log::info;
use trouble_host::{Controller, PacketPool, Stack};
use trouble_host::gatt::GattConnection;
use crate::comm::ble::BleGATTServer;

/// Example task to use the BLE notifier interface.
/// This task will notify the connected central of a counter value every 2 seconds.
/// It will also read the RSSI value every 2 seconds.
/// and will stop when the connection is closed by the central or an error occurs.
pub async fn custom_task<C: Controller, P: PacketPool>(
    server: &BleGATTServer<'_>,
    conn: &GattConnection<'_, '_, P>,
    stack: &Stack<'_, C, P>,
) {
    let mut tick: u8 = 0;
    let status_code = server.general_service.status_code;
    let wifi_get_status = server.general_service.wifi_get_status;
    loop {
        tick = tick.wrapping_add(1);
        info!("[custom_task] notifying connection of tick {}", tick);
        if status_code.notify(conn, &tick).await.is_err() {
            info!("[custom_task] error notifying connection");
            break;
        };

        if wifi_get_status.notify(conn, &tick).await.is_err() {
            info!("[custom_task] error notifying connection");
            break;
        };

        // read RSSI (Received Signal Strength Indicator) of the connection.
        if let Ok(_rssi) = conn.raw().rssi(stack).await {
            // info!("[custom_task] RSSI: {:?}", rssi);
        } else {
            info!("[custom_task] error getting RSSI");
            break;
        };
        Timer::after_secs(10).await;
    }
}