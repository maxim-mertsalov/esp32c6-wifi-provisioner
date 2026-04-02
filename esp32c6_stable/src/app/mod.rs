use embassy_futures::join::join;
use embassy_time::{Timer};
use log::{*};
use crate::board::Board;
use crate::comm::ble::{ble_task, init_gatt_server, ble_run};

pub mod state;
pub mod runner;

/// External Input Handler loop
pub async fn main_control_loop(board: Board, state: state::AppState) {
    let (
        server,
        mut peripheral,
        runner,
        stack
    ) = init_gatt_server(board.ble_controller);

    // Start the background BLE task
    // We join it with our logic loop
    join(ble_task(runner), async {
        loop {
            info!("IDLE: Waiting for 'Button Press' (Simulated)...");

            // SIMULATION: In a real app, you'd await a button interrupt here
            // For now, we just wait 5 seconds then "trigger" the Bluetooth mode
            Timer::after_secs(5).await;

            info!("Button Pressed! Bluetooth turning ON for 30 seconds...");
            
            // Block functions until the BLE task finishes (i.e. a phone connects and then disconnects, or 30 seconds pass)
            ble_run(server, &mut peripheral, stack, state).await;

            // The loop repeats, going back to the "IDLE" state
        }
    }).await;
}