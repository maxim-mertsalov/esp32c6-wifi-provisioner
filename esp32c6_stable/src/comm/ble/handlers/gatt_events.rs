use core::sync::atomic::Ordering;
use log::{info, warn};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent, ReadEvent, WriteEvent};
use trouble_host::{Error, PacketPool};
use crate::app::runner::RunnerCommand;
use crate::comm::ble::BleGATTServer;
use crate::comm::ble::utils::char_action::CharacteristicAction;
use crate::comm::wifi::models::{ENTIRE_SSID_PAGE_SIZE, MAX_PASSWORD_LEN, MAX_SSID_PER_PAGE, MAX_WIFI_CONNECTION_TYPE_SIZE, SERIALIZED_SSID_LEN};
use crate::prelude::AppState;

/// Stream Events until the connection closes.
///
/// This function will handle the GATT events and process them.
/// This is how we interact with read and write requests.
pub async fn gatt_events_task<P: PacketPool>(
    server: &BleGATTServer<'_>,
    conn: &GattConnection<'_, '_, P>,
    app_state: AppState,
) -> Result<(), Error> {
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                // Match Read/ Write events
                match &event {
                    GattEvent::Read(event) => {
                        match_read_events(event, server, app_state).await
                            .expect("error handling read event");
                    }
                    GattEvent::Write(event) => {
                        match_write_events(event, server, app_state).await;
                    },
                    GattEvent::Other(_) => {
                        info!("Other GATT event received");
                    }
                };

                // This step is also performed at drop(), but writing it explicitly is necessary
                // in order to ensure reply is sent.
                match event.accept() {
                    Ok(reply) => {
                        reply.send().await
                    },
                    Err(e) => warn!("[gatt] error sending response: {:?}", e),
                };
            }
            _ => {} // ignore other Gatt Connection Events
        }
    };
    info!("[gatt] disconnected: {:?}", reason);
    Ok(())
}

pub async fn match_read_events<P: PacketPool>(
    event: &ReadEvent<'_, '_, P>,
    server: &BleGATTServer<'_>,
    app_state: AppState
) -> Result<(), Error> {
    if let Some(action) = server.handle_action(event.handle()) {
        match action {
            CharacteristicAction::WifiGetStatus => {
                let data = app_state.wifi_status.load(Ordering::Relaxed);

                server.general_service.wifi_get_status.set(server, &data)?;
            }
            CharacteristicAction::WifiGetPagesCount => {
                let mut receiver = app_state.wifi_networks.receiver()
                    .expect("[gatt] error receiving gatt notification");

                let scan_data = receiver.try_get()
                    .expect("[gatt] error receiving receiver for wifi scan data").len();

                let optional_page = if scan_data % MAX_SSID_PER_PAGE == 0 { 0 } else { 1 };
                let pages = ((scan_data / MAX_SSID_PER_PAGE) + optional_page) as u8;

                server.general_service.wifi_get_pages_count.set(server, &pages)?;
            }
            CharacteristicAction::WifiGetPageData => {
                let mut receiver = app_state.wifi_networks.receiver()
                    .expect("[gatt] error receiving receiver for wifi scan data");

                let scan_data = receiver.try_get()
                    .expect("[gatt] error receiving wifi scan data");

                let current_page = app_state.current_page.load(Ordering::Relaxed) as usize;

                info!("[gatt] selected page: {}", current_page);

                let mut res = [0u8; ENTIRE_SSID_PAGE_SIZE];

                let from_wifi = current_page * MAX_SSID_PER_PAGE;
                let to_wifi = current_page * MAX_SSID_PER_PAGE + MAX_SSID_PER_PAGE;

                let mut index = 0;
                for i in from_wifi..to_wifi {
                    if let Some(scan_res) = scan_data.get(i) {
                        let serialized_scan = scan_res.as_bytes();

                        res[index..(index + SERIALIZED_SSID_LEN)].copy_from_slice(&serialized_scan);
                        index += SERIALIZED_SSID_LEN;
                    }
                }

                server.general_service.wifi_get_page_data.set(server, &res)?;
            }
            CharacteristicAction::StatusCode => {
                let data = app_state.status_code.load(Ordering::Relaxed);

                server.general_service.status_code.set(server, &data)?;
            }
            _ => {
                info!("incorrect gatt event action: {}", event.handle());
            }
        }
    }
    else {
        info!("[gatt] unknown gatt event action: {}", event.handle());
    }
    Ok(())
}


pub async fn match_write_events<P: PacketPool>(
    event: &WriteEvent<'_, '_, P>,
    server: &BleGATTServer<'_>,
    app_state: AppState
) {
    if let Some(action) = server.handle_action(event.handle()) {
        match action {
            CharacteristicAction::WifiScanCmd => {
                let sender = app_state.runner_command.sender();
                sender.send(RunnerCommand::WiFiStartScanning).await;
            }
            CharacteristicAction::WifiSelectPage => {
                let page = event.data()[0];

                let sender = app_state.runner_command.sender();
                sender.send(RunnerCommand::WiFiSelectScannedPage(page)).await;
            }
            CharacteristicAction::WifiSetSSIDIndex => {
                let index = event.data()[0];

                let sender = app_state.runner_command.sender();
                sender.send(RunnerCommand::WiFiSendSSIDIndex(index)).await;
            }
            CharacteristicAction::WifiSetPassword => {
                let password = event.data();
                let mut pass_layout = [0u8; MAX_PASSWORD_LEN];
                for i in 0..MAX_PASSWORD_LEN {
                    pass_layout[i] = *password.get(i).unwrap_or(&0u8);
                }

                let sender = app_state.runner_command.sender();
                sender.send(RunnerCommand::WiFiSendPassword(pass_layout)).await;
            }
            CharacteristicAction::WifiSetConnectionType => {
                let connection_type_raw = event.data();

                let mut connection_type = [0u8; MAX_WIFI_CONNECTION_TYPE_SIZE];
                for i in 0..MAX_WIFI_CONNECTION_TYPE_SIZE {
                    connection_type[i] = *connection_type_raw.get(i).unwrap_or(&0u8);
                }

                let sender = app_state.runner_command.sender();
                    sender.send(RunnerCommand::WifiSendConnectionType(connection_type)).await;
            }

            CharacteristicAction::WifiConnect => {
                let sender = app_state.runner_command.sender();
                sender.send(RunnerCommand::WifiTryConnect).await;
            }
            CharacteristicAction::WifiDisconnect => {
                let sender = app_state.runner_command.sender();
                sender.send(RunnerCommand::WifiTryDisconnect).await;
            }
            CharacteristicAction::WifiLocalTest => {
                let sender = app_state.runner_command.sender();
                sender.send(RunnerCommand::PingLocalNetwork).await;
            }
            CharacteristicAction::WifiGlobalTest => {
                let sender = app_state.runner_command.sender();
                sender.send(RunnerCommand::PingGlobalNetwork).await;
            }
            _ => {
                info!("incorrect gatt event action: {}", event.handle());
            }
        }
    }
    else {
        info!("[gatt] unknown gatt event action: {}", event.handle());
    }
}
