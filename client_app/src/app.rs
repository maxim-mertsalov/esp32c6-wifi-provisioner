use std::cmp::PartialEq;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::{anyhow, Context, Result};
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral as BlePeripheral};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use crate::models::{AuthMethod, WifiConnectionType, WifiCredentials, WifiScanResult};
use crate::services::Service;
use crate::utils::format_rssi_level;

pub struct App {
    pub peripheral: Arc<BlePeripheral>,
    pub services: Service,
}

impl App {
    pub fn new(peripheral: Arc<BlePeripheral>) -> Self {
        let services = Service::new(peripheral.clone());
        Self {
            peripheral,
            services,
        }
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let choices = vec![
                "Scan & Connect to Wi-Fi",
                "Get Wi-Fi Status",
                "Send global test",
                "Send local test",
                "Disconnect Wi-Fi",
                "Disconnect from BLE and exit",
            ];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Control menu")
                .items(&choices)
                .default(0)
                .interact()?;

            match selection {
                0 => {
                    if let Err(e) = self.scan_connect_handler().await{
                        println!("{}", e);
                    }
                },
                1 => {
                    let status = self.services.get_wifi_status().await?;
                    println!("Wi-Fi Status: {:?}", status);
                },
                2 => {
                    if let Err(e) = self.services.run_global_test().await {
                        println!("{}", e);
                    }
                    println!("Global test command sent.");
                },
                3 => {
                    if let Err(e) = self.services.run_local_test().await {
                        println!("{}", e);
                    }
                    println!("Local test command sent.");
                },
                4 => {
                    if let Err(e) = self.services.disconnect().await{
                        println!("{}", e);
                    }
                    println!("Wi-Fi disconnected.");
                },
                5 => {
                    if let Err(e) = self.services.disconnect().await{
                        println!("{}", e);
                    }
                    println!("Wi-Fi disconnected.");
                    exit(0);
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    async fn scan_connect_handler(&self) -> Result<()> {
        // Show menu with all networks and buttons rescan or return back

        loop {
            let networks = self.services.scan_networks().await?;

            let mut networks_display: Vec<String> = networks.iter()
                .map(|n| format!("{} {} (Auth: {:?})", format_rssi_level(n.rssi), n.ssid, n.auth_method))
                .collect();

            networks_display.insert(0, "⟳ Rescan".to_string());
            networks_display.insert(0, "← Back".to_string());

            let selected_network = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select operation or network:")
                .items(&networks_display)
                .default(0)
                .interact()?;

            if selected_network == 0 {
                return Ok(());
            }
            else if selected_network == 1 {
                continue;
            }
            else {
                if self.interact_with_network(selected_network - 2, &networks).await? {
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    async fn interact_with_network(&self, network_index: usize, networks: &Vec<WifiScanResult>) -> Result<bool> {
        let selected_network = networks[network_index].clone();
        let mut credentials = WifiCredentials::default();
        credentials.network_index = network_index;

        let mut menu_items = vec![
            "✎ Change connection type".to_string(),
            "↑ Connect to this network".to_string(),
            "← Back to networks list".to_string(),
        ];

        if selected_network.auth_method != AuthMethod::None {
            menu_items.insert(0, "+ Set Password".to_string());
        }

        loop {
            let header = format!("Selected Network: {} (Auth: {:?})\nCurrent Connection Type: {}", selected_network.ssid, selected_network.auth_method, credentials.connection_type);
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(&header)
                .items(&menu_items)
                .default(0)
                .interact()?;

            let real_selection = if selected_network.auth_method != AuthMethod::None {
                selection
            } else {
                selection + 1
            };

            match real_selection {
                0 => {
                    // set password
                    let password: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter Wi-Fi Password")
                        .interact_text()?;
                    credentials.password = password;
                },
                1 => {
                    // set connection type
                    let connection_type = ask_connection_type();
                    if let Err(e) = &connection_type {
                        println!("{}", *e);
                        continue;
                    }
                    let connection_type = connection_type?;
                    credentials.connection_type = connection_type;
                },
                2 => {
                    // connect
                    self.services.connect_to_wifi(credentials.clone()).await?;
                    return Ok(true);
                }
                3 => {
                    // back
                    return Ok(false);
                }
                _ => unreachable!(),
            }
        }

        Ok(false)
    }
}

fn ask_connection_type() -> Result<WifiConnectionType> {
    let menu_connection_type = vec![
        "DHCP".to_string(),
        "Static".to_string(),
        "DHCPv6".to_string(),
        "StaticV6".to_string(),
    ];
    let selected_connection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select connection type:")
        .items(&menu_connection_type)
        .default(0)
        .interact()?;

    let connection_type = match selected_connection {
        0 => WifiConnectionType::DHCP,
        1 => {
            let ip_raw: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter Static IP (e.g. 192.168.1.100)")
                .interact_text()?;

            let ip = Ipv4Addr::from_str(&ip_raw)
                .with_context(|| format!("Invalid IP address format: {}", ip_raw))?
                .octets();

            let subnet_mask_raw: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter Subnet Mask (e.g. 24)")
                .interact_text()?;

            let subnet = subnet_mask_raw.parse::<u8>()
                .context("Invalid Subnet Mask")?;

            let gateway_raw: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter Gateway (e.g. 192.168.1.1)")
                .interact_text()?;

            let gateway = Ipv4Addr::from_str(&gateway_raw)
                .context("Invalid Gateway Address format")?
                .octets();

            WifiConnectionType::Static { ip, subnet_mask: subnet, gateway }
        }
        2 => WifiConnectionType::DHCPv6,
        3 => {
            let ip_raw: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter Static IP (e.g. FF:FF:FF:FF:FF:FF:FF:FF)")
                .interact_text()?;

            let ip = Ipv6Addr::from_str(&ip_raw)
                .context("Invalid IP address format")?.octets();

            let prefix_length_raw: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter Prefix Length (e.g. 64)")
                .interact_text()?;
            let prefix_length = prefix_length_raw.parse::<u8>()
                .context("Invalid Prefix Length format")?;

            let gateway_raw: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter Gateway (e.g. FF:FF:FF:FF:FF:FF:FF:FF)")
                .interact_text()?;

            let gateway = Ipv6Addr::from_str(&gateway_raw)
                .context("Invalid Gateway Address format")?
                .octets();

            WifiConnectionType::StaticV6 { ip, prefix_length, gateway }
        }
        _ => unreachable!(),
    };

    Ok(connection_type)
}


