use alloc::string::ToString;
use core::str::FromStr;
use core::sync::atomic::Ordering;
use blocking_network_stack::Stack;
use embassy_futures::select::{select, Either};
use embassy_time::Timer;
use esp_hal::rng::Rng;
use esp_radio::wifi;
use esp_radio::wifi::{ClientConfig, ModeConfig, ScanConfig, WifiDevice};
use log::{info, warn};
use smoltcp::iface::{SocketSet, SocketStorage};
use static_cell::StaticCell;
use crate::comm::wifi::models::{WifiScanResult, WifiStatus};
use crate::comm::wifi::utils::create_interface;
use crate::prelude::AppState;
use crate::utils::timestamp_now;
use crate::comm::wifi::models::MAX_NETWORKS_ON_DEVICE;


pub enum WifiRunnerCommand {
    /// Try to connect to Wi-Fi via storage-saved credentials
    Connect, // Runtime default
    /// Scanning nearby networks and save to app_state
    Scan,
    /// Send ping to router
    PingLocal
}

#[embassy_executor::task]
pub async fn wifi_runner(
    app_state: AppState,
    mut wifi_controller: wifi::WifiController<'static>,
    mut wifi_device: wifi::WifiDevice<'static>,
    rng: Rng
) {
    // Buffers
    static RX_BUF: StaticCell<[u8; 1536]> = StaticCell::new();
    static TX_BUF: StaticCell<[u8; 1536]> = StaticCell::new();
    let rx_buffer = RX_BUF.init([0u8; 1536]);
    let tx_buffer = TX_BUF.init([0u8; 1536]);

    // Socket storage
    static SOCKET_ENTRIES: StaticCell<[SocketStorage; 3]> = StaticCell::new();
    let socket_set_entries = SOCKET_ENTRIES.init(Default::default());

    // Init network interface
    let now = timestamp_now() as i64;
    let interface = create_interface(
        &mut wifi_device,
        smoltcp::time::Instant::from_micros(now)
    );

    // Init socket set
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();
    socket_set.add(dhcp_socket);

    // Init stack
    let mut stack = blocking_network_stack::Stack::new(
        interface,
        wifi_device,
        socket_set,
        timestamp_now,
        rng.random()
    );
    info!("[WIFI_task]: Stack initialized");

    wifi_controller.start_async()
        .await
        .expect("WifiController crashed while starting");

    // stack.work();

    // let command = app_state.wifi_command.receive().await;

    // Connect to Wi-Fi network
    loop {
        let command_fut = app_state.wifi_command.receive();
        let timer_fut = Timer::after_millis(10);

        match select(command_fut, timer_fut).await {
            Either::First(command) => {
                match_wifi_command(
                    command,
                    &mut wifi_controller,
                    &mut stack,
                    app_state
                ).await;
            },
            Either::Second(_) => {}
        };

        let is_scanning = app_state.wifi_status.load(Ordering::Relaxed) == WifiStatus::Scanning as u8;
        if !is_scanning && wifi_controller.is_connected().unwrap_or(false) {
            stack.work();
        }
    }
}

pub async fn match_wifi_command(
    command: WifiRunnerCommand,
    wifi_controller: &mut wifi::WifiController<'static>,
    stack: &mut Stack<'_, WifiDevice<'static>>,
    app_state: AppState,
) {
    match command {
        WifiRunnerCommand::Connect => {
            let credentials = app_state.wifi_config.try_get()
                .unwrap_or(Default::default());

            app_state.wifi_status.store(WifiStatus::Connecting as u8, Ordering::Relaxed);

            if credentials.ssid.is_empty() {
                warn!("[WIFI_task]: Wifi SSID is empty");
                return;
            }

            info!("[WIFI_task]: Connecting...");

            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(credentials.ssid.as_str().to_string())
                    .with_password(credentials.password.as_str().to_string())
            );
            wifi_controller.set_config(&client_config)
                .expect("Failed to set Wi-Fi mode");

            if let Err(e) = wifi_controller.connect() {
                warn!("[WIFI_task]: Failed to connect to Wi-Fi mode: {}", e);
                app_state.wifi_status.store(WifiStatus::Error as u8, Ordering::Relaxed);
            } else {
                info!("[WIFI_task]: Connected to Wi-Fi network: {}", credentials.ssid);
                app_state.wifi_status.store(WifiStatus::Connected as u8, Ordering::Relaxed);
            }

        }
        WifiRunnerCommand::Scan => {
            info!("[WIFI_task]: Scanning...");

            app_state.wifi_status.store(WifiStatus::Scanning as u8, Ordering::Relaxed);

            match wifi_controller.scan_with_config_async(
                ScanConfig::default()
                    .with_max(MAX_NETWORKS_ON_DEVICE)
                    .with_show_hidden(true)
            ).await {
                Ok(networks) => {
                    let mut results: heapless::Vec<WifiScanResult, MAX_NETWORKS_ON_DEVICE> = heapless::Vec::new();

                    for net in networks.iter() {
                        results.push(WifiScanResult {
                            ssid: heapless::String::from_str(net.ssid.as_str()).unwrap_or(heapless::String::new()),
                            rssi: net.signal_strength,
                        }).expect("Failed save network to WifiScanResult");
                        info!("[WIFI_task]: found {} with signal: {} and auth: {:?}", net.ssid, net.signal_strength, net.auth_method);
                    }


                    app_state.wifi_networks.sender().send(results);
                    app_state.wifi_status.store(WifiStatus::Idle as u8, Ordering::Relaxed);
                    info!("[WIFI_task]: Found {} networks", networks.len());
                }
                Err(e) => {
                    warn!("[WIFI_task]: Scan error {:?}", e);
                    app_state.wifi_status.store(WifiStatus::Error as u8, Ordering::Relaxed);
                }
            }
            let heap_stats = esp_alloc::HEAP.stats();
            info!("[WIFI_task]:\n{}", heap_stats);
        }
        WifiRunnerCommand::PingLocal => {
            if wifi_controller.is_connected().unwrap_or(false) && stack.is_iface_up() {
                info!("[WIFI_task]: Online! Ip: {:?}", stack.get_ip_info());
                app_state.wifi_status.store(WifiStatus::Connected as u8, Ordering::Relaxed);
            } else {
                app_state.wifi_status.store(WifiStatus::ErrorNoConnection as u8, Ordering::Relaxed);
            }
        }
    }
}