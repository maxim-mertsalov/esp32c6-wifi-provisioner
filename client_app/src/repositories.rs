use std::sync::Arc;
use anyhow::{anyhow, Result};
use btleplug::api::{Characteristic, Peripheral, WriteType};
use btleplug::platform::{Peripheral as BlePeripheral};
use uuid::Uuid;

pub struct Repositories {
    peripheral: Arc<BlePeripheral>
}

impl Repositories {
    pub fn new(peripheral: Arc<BlePeripheral>) -> Self { Self { peripheral } }
    pub async fn find_char(&self, uuid: Uuid) -> Result<Characteristic> {
        self.peripheral.characteristics()
            .into_iter()
            .find(|c| c.uuid == uuid)
            .ok_or_else(|| anyhow!("Characteristic {} not found!", uuid))
    }

    pub async fn send(&self, uuid: Uuid, val: &[u8]) -> Result<()> {
        let c = self.find_char(uuid).await?;
        self.peripheral.write(&c, val, WriteType::WithResponse).await?;
        println!("Command sent {}", uuid);
        Ok(())
    }

    pub async fn read(&self, uuid: Uuid) -> Result<Vec<u8>> {
        let c = self.find_char(uuid).await?;
        let data = self.peripheral.read(&c).await?;
        Ok(data)
    }
}