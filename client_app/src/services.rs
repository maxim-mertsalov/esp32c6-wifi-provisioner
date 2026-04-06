use std::sync::Arc;
use anyhow::{anyhow, Result};
use btleplug::platform::{Peripheral as BlePeripheral};
use crate::repositories::Repositories;
use crate::ble_constants;
use crate::models::{WifiCredentials, WifiScanResult};

pub struct Service {
    peripheral: Arc<BlePeripheral>,
    repos: Repositories
}

impl Service {
    pub fn new(peripheral: Arc<BlePeripheral>) -> Self {
        let repos = Repositories::new(peripheral.clone());
        Self { peripheral, repos }
    }

    pub async fn scan_networks(&self) -> Result<Vec<WifiScanResult>> {
        self.repos.send(ble_constants::WIFI_SCAN_CMD, &[1u8]).await?;

        println!("Scanning WiFi Networks...");
        loop {
            let wifi_status = *self.repos.read(ble_constants::WIFI_GET_STATUS).await?.get(0).unwrap();

            if wifi_status == 1 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            else if wifi_status == 0 {
                println!("Scan complete!");
                break;
            }
            else {
                return Err(anyhow!("Error: {}", wifi_status));
            }
        }

        let pages_found = *self.repos.read(ble_constants::WIFI_GET_PAGES_COUNT).await?.get(0).unwrap();

        let mut all_networks = Vec::new();

        println!("Found {} pages of networks", pages_found);
        for page in 0..pages_found {
            self.repos.send(ble_constants::WIFI_SELECT_PAGE, &[page]).await?;
            let data = self.repos.read(ble_constants::WIFI_GET_PAGE_DATA).await?;
            let mut networks = WifiScanResult::from_bytes(&data);

            all_networks.append(&mut networks);
        }


        Ok(all_networks)
    }

    pub async fn connect_to_wifi(
        &self,
        wifi_credentials: WifiCredentials,
    ) -> Result<()> {
        // set index of network
        let wifi_index_raw = wifi_credentials.network_index as u8;
        self.repos.send(ble_constants::WIFI_SET_SSID_INDEX, &[wifi_index_raw]).await?;

        // set connection type
        let connection_type_raw = wifi_credentials.connection_type.as_bytes();
        self.repos.send(ble_constants::WIFI_SET_CONNECTION_TYPE, &connection_type_raw).await?;

        // set password
        let password = wifi_credentials.password.as_bytes();
        self.repos.send(ble_constants::WIFI_SET_PASSWORD, &password).await?;

        // connect
        self.repos.send(ble_constants::WIFI_CONNECT, &[1u8]).await?;

        // handle wifi_status
        println!("Connecting to WiFi...");
        loop {
            let wifi_status = *self.repos.read(ble_constants::WIFI_GET_STATUS).await?.get(0).unwrap();

            if wifi_status == 2 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            else if wifi_status < 100 {
                println!("Connected! Status code: {}", wifi_status);
                break;
            }
            else {
                return Err(anyhow!("Error: {}", wifi_status));
            }
        }

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        println!("Disconnecting...");
        self.repos.send(ble_constants::WIFI_DISCONNECT, &[1u8]).await?;

        Ok(())
    }

    pub async fn get_wifi_status(&self) -> Result<u8> {
        self.repos.read(ble_constants::WIFI_GET_STATUS).await?
            .get(0)
            .copied()
            .ok_or_else(|| anyhow!("Failed to read WiFi status"))
    }

    pub async fn run_local_test(&self) -> Result<()> {
        self.repos.send(ble_constants::WIFI_LOCAL_TEST, &[1u8]).await?;

        println!("Running local test...");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let wifi_status = *self.repos.read(ble_constants::WIFI_GET_STATUS).await?.get(0).unwrap();
        if wifi_status < 100 {
            println!("Success: {}", wifi_status);
        }
        else {
            return Err(anyhow!("Error: {}", wifi_status));
        }

        Ok(())
    }

     pub async fn run_global_test(&self) -> Result<()> {
         self.repos.send(ble_constants::WIFI_GLOBAL_TEST, &[1u8]).await?;

         println!("Running global test...");
         tokio::time::sleep(std::time::Duration::from_secs(3)).await;

         let wifi_status = *self.repos.read(ble_constants::WIFI_GET_STATUS).await?.get(0).unwrap();
         if wifi_status < 100 {
             println!("Success: {}", wifi_status);
         }
         else {
             return Err(anyhow!("Error: {}", wifi_status));
         }

        Ok(())
    }
}