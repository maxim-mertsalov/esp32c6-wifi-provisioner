use core::sync::atomic::Ordering;
use heapless::{String, Vec};
use log::{*};
use crate::comm::wifi::models::{WifiConnectionType, WifiCredentials, WifiStatus, MAX_PASSWORD_LEN, MAX_WIFI_CONNECTION_TYPE_SIZE};
use crate::comm::wifi::runner::WifiRunnerCommand;
use crate::prelude::AppState;

#[derive(Copy, Clone)]
pub enum RunnerCommand {
    WiFiStartScanning,
    WiFiSelectScannedPage(u8),

    WiFiSendSSIDIndex(u8),
    WiFiSendPassword([u8; MAX_PASSWORD_LEN]),
    WifiSendConnectionType([u8; MAX_WIFI_CONNECTION_TYPE_SIZE]),

    WifiTryConnect,
    WifiTryDisconnect,

    PingLocalNetwork,
    PingGlobalNetwork,

    SendServerUrl([u8; 64]),
    SendGetRequest,
}

/// Main control loop, that switches commands between other threads
#[embassy_executor::task]
pub async fn runner_task(app_state: AppState) {
    info!("Starting runner task");

    let receiver = app_state.runner_command.receiver();

    loop {
        // REQUIRED! If you don't await this, function will take entire processor
        let command = receiver.receive().await;

        match command {
            RunnerCommand::WiFiStartScanning => {
                info!("Wifi start scanning");
                app_state.wifi_command.sender()
                    .send(WifiRunnerCommand::Scan).await;
            }
            RunnerCommand::WiFiSelectScannedPage(page) => {
                info!("WiFi select scanned page: {}", page);
                app_state.current_page.store(page, Ordering::Relaxed);
            }
            RunnerCommand::WiFiSendSSIDIndex(index) => {
                info!("WiFi send SSID index: {}", index);
                let wifi_networks = app_state.wifi_networks.try_get()
                    .unwrap_or(Vec::new());

                if wifi_networks.is_empty() {
                    app_state.wifi_status.store(WifiStatus::ErrorNoScannedNetworks as u8, Ordering::Relaxed);
                } else {
                    let scan = &wifi_networks.get(index as usize);

                    if scan.is_none() {
                        warn!("Incorrect WiFi index: {}", index);
                        continue;
                    }

                    let scan = scan.unwrap();

                    let ssid = &scan.ssid;
                    info!("Selected WiFi SSID: {}", ssid);

                    let old_wifi_config = app_state.wifi_config.try_get()
                        .unwrap_or_default();

                    app_state.wifi_config.sender()
                        .send(WifiCredentials {
                            ssid: ssid.clone(), // todo: too heavy
                            password: old_wifi_config.password,
                            auth_method: scan.auth_method,
                            connection_type: WifiConnectionType::DHCP
                        })
                }
            }
            RunnerCommand::WiFiSendPassword(password) => {
                info!("WiFi send password: {:?}", password);
                let old_config = app_state.wifi_config.try_get()
                    .unwrap_or_default();

                let pass_vec: Vec<u8, MAX_PASSWORD_LEN> = Vec::from_slice(&password)
                    .unwrap_or_default();

                let password = String::from_utf8(pass_vec)
                    .unwrap_or_default();

                info!("WiFi send password: {:?}", password);

                app_state.wifi_config.sender()
                    .send(WifiCredentials {
                        ssid: old_config.ssid,
                        password,
                        auth_method: old_config.auth_method,
                        connection_type: old_config.connection_type,
                    })
            }
            RunnerCommand::WifiSendConnectionType(connection_type) => {
                info!("WiFi send connection type: {:?}", connection_type);
                let old_config = app_state.wifi_config.try_get()
                    .unwrap_or_default();

                let connection_type = WifiConnectionType::from_bytes(connection_type);

                info!("WiFi send connection type: {:?}", connection_type);

                app_state.wifi_config.sender()
                    .send(WifiCredentials {
                        ssid: old_config.ssid,
                        password: old_config.password,
                        auth_method: old_config.auth_method,
                        connection_type,
                    })
            }

            RunnerCommand::WifiTryConnect => {
                info!("Wifi try connect");
                app_state.wifi_command.sender()
                    .send(WifiRunnerCommand::Connect).await;
            }
            RunnerCommand::WifiTryDisconnect => {
                info!("Wifi try disconnect");
                app_state.wifi_command.sender()
                    .send(WifiRunnerCommand::Disconnect).await;
            }
            RunnerCommand::SendServerUrl(_) => {
                info!("Sending server URL");
            }
            RunnerCommand::SendGetRequest => {
                info!("Sending get request");
            }
            RunnerCommand::PingLocalNetwork => {
                info!("Testing connection");
                app_state.wifi_command.sender()
                    .send(WifiRunnerCommand::PingLocal).await;
            },
            RunnerCommand::PingGlobalNetwork => {
                info!("Testing connection");
                app_state.wifi_command.sender()
                    .send(WifiRunnerCommand::PingGlobal).await;
            }
        }

    }
}
