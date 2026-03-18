use log::{*};
use crate::app::state::AppStateCommand;
use crate::prelude::AppState;

#[embassy_executor::task]
pub async fn runner_task(app_state: AppState) {
    info!("Starting runner task");

    let mut receiver = app_state.current_command.receiver()
        .expect("No command receivers");

    loop {
        // REQUIRED! If you don't await this, function will take entire processor
        let command_opt = receiver.changed().await;

        if let Some(command) = command_opt {
            match command {
                AppStateCommand::WiFiStartScanning => {
                    info!("Wifi start scanning");
                }
                AppStateCommand::WiFiSelectScannedPage(page) => {
                    info!("WiFi select scanned page: {}", page);
                }
                AppStateCommand::WiFiSendSSIDIndex(index) => {
                    info!("WiFi send SSID index: {}", index);
                }
                AppStateCommand::WiFiSendPassword(password) => {
                    info!("WiFi send password: {:?}", password);
                }
                AppStateCommand::SendServerUrl(_) => {
                    info!("Sending server URL");
                }
                AppStateCommand::SendGetRequest => {
                    info!("Sending get request");
                }
                AppStateCommand::TestConnection => {
                    info!("Testing connection");
                },
            }
        }

    }
}
