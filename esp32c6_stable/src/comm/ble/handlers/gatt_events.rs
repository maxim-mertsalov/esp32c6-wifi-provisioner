use core::sync::atomic::Ordering;
use log::{info, warn};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent, ReadEvent, WriteEvent};
use trouble_host::{Error, PacketPool};
use crate::app::state::AppStateCommand;
use crate::comm::ble::BleGATTServer;
use crate::comm::ble::utils::char_action::CharacteristicAction;
use crate::comm::wifi::models::{MAX_SSID_PER_PAGE, SHORT_SSID_LEN};
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
                        match_read_events(event, server, app_state).await;
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

pub async fn match_read_events<P: PacketPool>(event: &ReadEvent<'_, '_, P>, server: &BleGATTServer<'_>, app_state: AppState) {
    if let Some(action) = server.handle_action(event.handle()) {
        match action {
            CharacteristicAction::WifiGetStatus => {
                let data = app_state.wifi_status.load(Ordering::Relaxed);

                server.general_service.wifi_get_status.set(server, &data)
                    .expect("[gatt] error getting status");
            }
            CharacteristicAction::WifiGetPagesCount => {
                let mut receiver = app_state.wifi_networks.receiver()
                    .expect("[gatt] error receiving gatt notification");

                let scan_data = receiver.try_get()
                    .expect("[gatt] error receiving receiver for wifi scan data").len();
                let pages = (scan_data / MAX_SSID_PER_PAGE) as u8;

                server.general_service.wifi_get_pages_count.set(server, &pages)
                    .expect("[gatt] error getting status");
            }
            CharacteristicAction::WifiGetPageData => {
                let mut receiver = app_state.wifi_networks.receiver()
                    .expect("[gatt] error receiving receiver for wifi scan data");

                let scan_data = receiver.try_get()
                    .expect("[gatt] error receiving wifi scan data");

                let current_page = app_state.current_page.load(Ordering::Relaxed) as usize;

                let mut res = [0u8; MAX_SSID_PER_PAGE * (SHORT_SSID_LEN + 1)];

                let from_wifi = current_page * MAX_SSID_PER_PAGE;
                let to_wifi = current_page * MAX_SSID_PER_PAGE + MAX_SSID_PER_PAGE;

                let mut index = 0;
                for i in from_wifi..to_wifi {
                    if let Some(scan_res) = scan_data.get(i) {
                        let new_len = SHORT_SSID_LEN.min(scan_res.ssid.len());
                        let short_ssid = scan_res.ssid.as_bytes();

                        res[index..(index + new_len)].copy_from_slice(short_ssid);
                        index += SHORT_SSID_LEN;
                        res[index] = scan_res.rssi;
                        index += 1;
                    }
                }

                server.general_service.wifi_get_page_data.set(server, &res)
                    .expect("[gatt] error getting status");
            }
            CharacteristicAction::StatusCode => {
                let data = app_state.status_code.load(Ordering::Relaxed);

                server.general_service.status_code.set(server, &data)
                    .expect("[gatt] error getting status");
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


pub async fn match_write_events<P: PacketPool>(event: &WriteEvent<'_, '_, P>, server: &BleGATTServer<'_>, app_state: AppState) {
    if let Some(action) = server.handle_action(event.handle()) {
        match action {
            CharacteristicAction::WifiScanCmd => {
                let sender = app_state.current_command.sender();
                sender.send(Some(AppStateCommand::WiFiStartScanning));
            }
            CharacteristicAction::WifiSelectPage => {
                let page = event.data()[0];
                let sender = app_state.current_command.sender();
                sender.send(Some(AppStateCommand::WiFiSelectScannedPage(page)));
                info!("Received WifiSelectPage command with page: {:?}", page);
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
