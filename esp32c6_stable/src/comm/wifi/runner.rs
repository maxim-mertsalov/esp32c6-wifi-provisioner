use alloc::string::ToString;
use core::str::FromStr;
use core::sync::atomic::Ordering;
use embassy_futures::select::{select, Either};
use embassy_net::{Config, Runner, Stack, StackResources, tcp::TcpSocket, icmp::{IcmpSocket, PacketMetadata}};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use esp_radio::wifi;
use esp_radio::wifi::{ClientConfig, ModeConfig, ScanConfig, ScanTypeConfig, WifiDevice};
use log::{*};
use static_cell::StaticCell;
use crate::comm::wifi::models::{WifiScanResult, WifiStatus};
use crate::prelude::AppState;
use crate::comm::wifi::models::MAX_NETWORKS_ON_DEVICE;
use crate::comm::wifi::utils::{apply_ip_config, nslookup};

pub enum WifiRunnerCommand {
    /// Try to connect to Wi-Fi via storage-saved credentials
    Connect, // Runtime default
    /// Disconnect
    Disconnect,
    /// Scanning nearby networks and save to app_state
    Scan,
    /// Send ping to router
    PingLocal,
    /// Send ping to google.com/generate_204
    PingGlobal,
}

#[embassy_executor::task]
pub async fn wifi_runner(
    app_state: AppState,
    mut wifi_controller: wifi::WifiController<'static>,
    wifi_device: WifiDevice<'static>,
    rng: Rng,
    spawner: embassy_executor::Spawner
) {
    // Disable Low Power Mode
    wifi_controller.set_power_saving(wifi::PowerSaveMode::None)
        .expect("[WIFI_task] Failed to set power saving");

    // Setup DHCP
    let dhcp_config = embassy_net::DhcpConfig::default();
    let config = Config::dhcpv4(dhcp_config);
    let seed = rng.random() as u64;

    // Resources
    static STACK_RESOURCES: StaticCell<StackResources<6>> = StaticCell::new(); // 1 DHCP, 2 TCP, 2 ICMP, 1 DNS
    let resources = STACK_RESOURCES.init(StackResources::new());

    // Stack
    let (stack, runner) = embassy_net::new(
        wifi_device,
        config,
        resources,
        seed
    );
    static STACK_STORAGE: StaticCell<Stack<'static>> = StaticCell::new();
    let stack = &*STACK_STORAGE.init(stack);

    info!("[WIFI_task]: Stack initialized");
    info!("[WIFI_task]: Starting Wi-Fi runner");

    // background stack task
    spawner.must_spawn(net_task(runner));

    wifi_controller.start_async()
        .await
        .expect("WifiController crashed while starting");
    info!("[WIFI_task]: Wi-Fi controller running");

    let mut retry_count = 0;
    let mut connection_started_at: Option<embassy_time::Instant> = None;

    // Connect to Wi-Fi network
    loop {
        let command_fut = app_state.wifi_command.receive();
        let timer_fut = Timer::after_millis(100);

        check_connecting_status(
            app_state,
            &mut wifi_controller,
            &stack,
            &mut connection_started_at,
            &mut retry_count
        ).await;

        match select(command_fut, timer_fut).await {
            Either::First(command) => {
                match command {
                    WifiRunnerCommand::Connect => {
                        handle_connect(
                            app_state,
                            &mut wifi_controller,
                            &stack,
                            &mut connection_started_at,
                            &mut retry_count
                        ).await;
                    }
                    WifiRunnerCommand::Disconnect => {
                        handle_disconnect(
                            app_state,
                            &mut wifi_controller,
                        ).await;
                    }
                    WifiRunnerCommand::Scan => {
                        handle_scan(
                            app_state,
                            &mut wifi_controller,
                        ).await;
                    }
                    WifiRunnerCommand::PingLocal => {
                        handle_test_router(
                            app_state,
                            &stack
                        ).await;
                    }
                    WifiRunnerCommand::PingGlobal => {
                        handle_test_global(
                            app_state,
                            &stack
                        ).await;
                    }
                }
            },
            Either::Second(_) => {
                if !wifi_controller.is_connected().unwrap_or(false) && connection_started_at.is_none() {
                    app_state.wifi_status.store(WifiStatus::Idle as u8, Ordering::Relaxed);
                }
            }
        };
    }
}

async fn check_connecting_status(
    app_state: AppState,
    wifi_controller: &mut wifi::WifiController<'static>,
    stack: &Stack<'static>,
    connection_started_at: &mut Option<embassy_time::Instant>,
    retry_count: &mut u8
) {
    const MAX_RETRIES: u8 = 5;
    const CONN_TIMEOUT_MS: u64 = 10_000; // 10 s.

    let wifi_status = app_state.wifi_status.load(Ordering::Relaxed);

    if wifi_status == WifiStatus::Connecting as u8 {
        if wifi_controller.is_connected().unwrap_or(false) {
            if let Some(config) = stack.config_v4() {
                info!("[WIFI_task] IP assigned: {:?}", config.address);
                app_state.wifi_status.store(WifiStatus::Connected as u8, Ordering::Relaxed);
                *connection_started_at = None;
                *retry_count = 0;
            } else {
                debug!("[WIFI_task] WiFi Link UP, waiting for IP...");
            }
        } else if let Some(start_time) = connection_started_at {
            // Check for timeout
            if start_time.elapsed().as_millis() > CONN_TIMEOUT_MS {
                warn!("[WIFI_task]: Connection attempt {} timed out", *retry_count + 1);

                *retry_count += 1;
                if *retry_count < MAX_RETRIES {
                    info!("[WIFI_task]: Retrying... ({} of {})", *retry_count + 1, MAX_RETRIES);
                    // Reset driver
                    let _ = wifi_controller.disconnect();
                    let _ = wifi_controller.connect();
                    *connection_started_at = Some(embassy_time::Instant::now());
                } else {
                    error!("[WIFI_task]: Max retries reached. Giving up.");
                    app_state.wifi_status.store(WifiStatus::Error as u8, Ordering::Relaxed);
                    *connection_started_at = None;
                    let _ = wifi_controller.disconnect();
                }
            }
        }
    }
}

async fn handle_connect(
    app_state: AppState,
    wifi_controller: &mut wifi::WifiController<'static>,
    stack: &Stack<'static>,
    connection_started_at: &mut Option<embassy_time::Instant>,
    retry_count: &mut u8
) {
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
            .with_auth_method(credentials.auth_method)
    );
    // TODO: EapClient for enterprise networks
    let mode_res = wifi_controller.set_config(&client_config);
    if let Err(e) = mode_res {
        warn!("[WIFI_task]: Failed to set Wi-Fi mode: {}", e);
        app_state.wifi_status.store(WifiStatus::Error as u8, Ordering::Relaxed);
        return;
    }

    // Set connection type e.g. DHCP or Static
    let connection_type = app_state.wifi_config.try_get()
        .unwrap_or_default()
        .connection_type;

    apply_ip_config(stack, connection_type).await;

    // Connecting
    if let Err(e) = wifi_controller.connect() {
        warn!("[WIFI_task]: Could not start connection: {}", e);
        app_state.wifi_status.store(WifiStatus::Error as u8, Ordering::Relaxed);
    } else {
        info!("[WIFI_task]: Connection attempt started to {}...", credentials.ssid);
        *connection_started_at = Some(embassy_time::Instant::now());
        *retry_count = 0;
    }
}


async fn handle_disconnect(
    app_state: AppState,
    wifi_controller: &mut wifi::WifiController<'static>,
) {
    info!("[WIFI_task]: Disconnecting...");

    if let Err(e) = wifi_controller.disconnect() {
        warn!("[WIFI_task]: Failed to disconnect: {}", e);
        app_state.wifi_status.store(WifiStatus::Error as u8, Ordering::Relaxed);
    } else {
        info!("[WIFI_task]: Disconnected from Wi-Fi network");
        app_state.wifi_status.store(WifiStatus::Idle as u8, Ordering::Relaxed);
    }
}


async fn handle_scan(
    app_state: AppState,
    wifi_controller: &mut wifi::WifiController<'static>,
){
    info!("[WIFI_task]: Scanning...");

    app_state.wifi_status.store(WifiStatus::Scanning as u8, Ordering::Relaxed);

    match wifi_controller.scan_with_config_async(
        ScanConfig::default()
            .with_max(MAX_NETWORKS_ON_DEVICE)
            .with_show_hidden(true)
            .with_scan_type(ScanTypeConfig::Active{
                min: core::time::Duration::from_millis(120),
                max: core::time::Duration::from_millis(150),
            })
    ).await {
        Ok(networks) => {
            let mut results: heapless::Vec<WifiScanResult, MAX_NETWORKS_ON_DEVICE> = heapless::Vec::new();

            for net in networks.iter() {
                if net.ssid.is_empty() {
                    continue;
                }

                let ssid = heapless::String::from_str(net.ssid.as_str()).unwrap_or_default();
                let auth_method = net.auth_method.unwrap_or_default();

                if let Err(e) = results.push(WifiScanResult {
                    ssid,
                    rssi: net.signal_strength,
                    auth_method
                }) { warn!("[WIFI_task]: Failed to push scan result: {:?}", e); };
                info!("[WIFI_task]: found {} with signal: {} and auth: {:?}", net.ssid, net.signal_strength, net.auth_method);
            }

            let count_net = results.len();


            app_state.wifi_networks.sender().send(results);
            app_state.wifi_status.store(WifiStatus::Idle as u8, Ordering::Relaxed);
            info!("[WIFI_task]: Found {} networks", count_net);
        }
        Err(e) => {
            warn!("[WIFI_task]: Scan error {:?}", e);
            app_state.wifi_status.store(WifiStatus::Error as u8, Ordering::Relaxed);
        }
    }
    let heap_stats = esp_alloc::HEAP.stats();
    info!("[WIFI_task]:\n{}", heap_stats);
}


async fn handle_test_global(
    app_state: AppState,
    stack: &Stack<'static>
) {
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(5)));

    // Google IP (142.251.38.142)
    let remote_endpoint = (nslookup(&stack, "google.com").await
        .unwrap_or(embassy_net::IpAddress::v4(142, 251, 38, 142)), 80);

    info!("[WIFI_task]: Pinging internet...");

    if let Err(e) = socket.connect(remote_endpoint).await {
        warn!("[WIFI_task]: Could not connect to remote endpoint: {:?}", e);
        app_state.wifi_status.store(WifiStatus::ErrorNoInternet as u8, Ordering::Relaxed);
        return;
    }

    let req = b"GET /generate_204 HTTP/1.1\r\nHost: google.com\r\nConnection: close\r\n\r\n";
    if socket.write(req).await.is_ok() {
        let mut response_buf = [0u8; 128];
        if let Ok(n) = socket.read(&mut response_buf).await {
            if n > 0 && core::str::from_utf8(&response_buf[..n]).unwrap_or("").contains("204") {
                info!("[WIFI_task]: Internet OK!");
                app_state.wifi_status.store(WifiStatus::ConnectedWithInterner as u8, Ordering::Relaxed);
            }
        }
        else {
            warn!("[WIFI_task]: Error while reading internet response");
        }
    } else {
        warn!("[WIFI_task]: Error while writing to socket");
    }
}

async fn handle_test_router(
    app_state: AppState,
    stack: &Stack<'static>
) {
    // Check if Wi-Fi configured
    let Some(config) = stack.config_v4() else {
        warn!("[WIFI_task]: No IP config yet, cannot ping local");
        return;
    };

    info!("[WIFI_task]: Config: {:?}", config);

    let Some(gateway) = config.gateway else {
        warn!("[WIFI_task]: No gateway found in DHCP config");
        return;
    };
    // let gateway = gateway.octets();

    info!("[WIFI_task]: Pinging gateway: {:?}", gateway);

    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut rx_payload = [0u8; 256];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_payload = [0u8; 256];

    let icmp = IcmpSocket::new(
        *stack,
        &mut rx_meta,
        &mut rx_payload,
        &mut tx_meta,
        &mut tx_payload,
    );

    // Send Echo Request
    let mut echo_payload = [0u8; 8];
    echo_payload[0] = 8; // Type: Echo Request
    echo_payload[1] = 0;
    echo_payload[4] = 0xBE;  // Identifier
    echo_payload[5] = 0xEF;
    echo_payload[6] = 0x00;  // Sequence number
    echo_payload[7] = 0x01;
    if let Err(e) = icmp.send_to(&echo_payload, gateway).await {
        warn!("[WIFI_task]: Failed to send ICMP request: {:?}", e);
        app_state.wifi_status.store(WifiStatus::ErrorNoConnection as u8, Ordering::Relaxed);
        return;
    }

    // Wait for response
    match embassy_time::with_timeout(Duration::from_secs(2), icmp.recv_from_with(|d| {
        info!("[WIFI_task]: Pong received from {:?} with: {:?}", d.1, d.0);
    })).await {
        Ok(Ok(_)) => {
            info!("[WIFI_task]: Pong received");
            app_state.wifi_status.store(WifiStatus::Connected as u8, Ordering::Relaxed);
        }
        Ok(Err(e)) => {
            warn!("[WIFI_task]: ICMP error: {:?}", e);
            app_state.wifi_status.store(WifiStatus::ErrorNoConnection as u8, Ordering::Relaxed);
        }
        Err(_) => {
            warn!("[WIFI_task]: Local ping timeout (Gateway unreachable)");
            app_state.wifi_status.store(WifiStatus::ErrorNoConnection as u8, Ordering::Relaxed);
        }
    }

}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}