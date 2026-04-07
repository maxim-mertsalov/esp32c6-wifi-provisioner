pub mod ble_constants;
pub mod app;
pub mod utils;
pub mod state;
pub mod services;
pub mod repositories;
pub mod models;

use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Context, Result};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::time::sleep;
use crate::app::App;
use crate::utils::select_peripheral;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = Manager::new().await?;
    let adapter = manager.adapters().await?
        .into_iter().next()
        .ok_or_else(|| anyhow!("Bluetooth adapter not found"))?;

    println!("Scanning for devices...");
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(3)).await;

    let peripherals = adapter.peripherals().await?;
    if peripherals.is_empty() {
        return Err(anyhow!("Device not found"));
    }

    // Selecting device
    let peripheral = if peripherals.len() == 1 {
        let p = peripherals.into_iter().next().unwrap();
        println!("Found one device, connecting...");
        p
    } else {
        select_peripheral(peripherals).await?
    };

    peripheral.connect().await.context("Could not connect to Bluetooth device")?;
    peripheral.discover_services().await.context("Error while discovering services")?;

    let services = peripheral.services();

    if services.is_empty() {
        return Err(anyhow!("No services found on the device"));
    }

    let mut discovered = false;
    for service in services.iter() {
        if service.uuid == ble_constants::SERVICE_UUID {
            // println!("Found service: {}", service.uuid);
            discovered = true;
        }
    }
    if !discovered {
        return Err(anyhow!("Bluetooth service not match"));
    }

    let shared_peripheral = Arc::new(peripheral);

    let app = App::new(shared_peripheral.clone());
    app.run().await?;

    Ok(())
}
