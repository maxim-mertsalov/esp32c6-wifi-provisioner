use core::sync::atomic::Ordering;
use log::{*};
use crate::comm::wifi::models::MAX_PASSWORD_LEN;
use crate::comm::wifi::runner::WifiRunnerCommand;
use crate::prelude::AppState;

#[derive(Copy, Clone)]
pub enum RunnerCommand {
    WiFiStartScanning,
    WiFiSelectScannedPage(u8),

    WiFiSendSSIDIndex(u8),
    WiFiSendPassword([u8; MAX_PASSWORD_LEN]),

    WifiTryConnect,

    SendServerUrl([u8; 64]),
    SendGetRequest,

    TestConnection,
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
            }
            RunnerCommand::WiFiSendPassword(password) => {
                info!("WiFi send password: {:?}", password);
            }
            RunnerCommand::SendServerUrl(_) => {
                info!("Sending server URL");
            }
            RunnerCommand::SendGetRequest => {
                info!("Sending get request");
            }
            RunnerCommand::TestConnection => {
                info!("Testing connection");
            },
            _ => {}
        }

    }
}
