use portable_atomic::{AtomicU8};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;
use heapless::Vec;
use static_cell::StaticCell;
use crate::comm::wifi::models::{WifiCredentials, WifiScanResult, MAX_NETWORKS_ON_DEVICE};


#[derive(Copy, Clone)]
pub struct AppState {
    pub wifi_config: &'static Watch<CriticalSectionRawMutex, Option<WifiCredentials>, 4>,
    pub wifi_status: &'static AtomicU8,
    pub wifi_networks: &'static Watch<CriticalSectionRawMutex, Vec<WifiScanResult, MAX_NETWORKS_ON_DEVICE>, 4>,
    pub current_page: &'static AtomicU8,

    pub server_url: &'static Watch<CriticalSectionRawMutex, Option<heapless::String<64>>, 4>,

    pub status_code: &'static AtomicU8,
    pub current_command:  &'static Watch<CriticalSectionRawMutex, Option<AppStateCommand>, 4>,
}

#[derive(Copy, Clone)]
pub enum AppStateCommand {
    WiFiStartScanning,
    WiFiSelectScannedPage(u8),
    WiFiSendSSIDIndex(u8),
    WiFiSendPassword([u8; 64]),

    SendServerUrl([u8; 64]),
    SendGetRequest,

    TestConnection,
}

impl Default for AppState {
    fn default() -> Self {
        static WIFI_CONFIG: StaticCell<Watch<CriticalSectionRawMutex, Option<WifiCredentials>, 4>> = StaticCell::new();
        static WIFI_STATUS: StaticCell<AtomicU8> = StaticCell::new();
        static WIFI_NETWORKS: StaticCell<Watch<CriticalSectionRawMutex, Vec<WifiScanResult, MAX_NETWORKS_ON_DEVICE>, 4>> = StaticCell::new();
        static CURRENT_PAGE_ID: StaticCell<AtomicU8> = StaticCell::new();
        static SERVER_URL: StaticCell<Watch<CriticalSectionRawMutex, Option<heapless::String<64>>, 4>> = StaticCell::new();

        static STATUS_CODE: StaticCell<AtomicU8> = StaticCell::new();
        static CURRENT_COMMAND: StaticCell<Watch<CriticalSectionRawMutex, Option<AppStateCommand>, 4>> = StaticCell::new();

        AppState {
            wifi_config: WIFI_CONFIG.init(Watch::new_with(None)),
            wifi_status: WIFI_STATUS.init(AtomicU8::new(0)),
            wifi_networks: WIFI_NETWORKS.init(Watch::new_with(Vec::new())),
            current_page: CURRENT_PAGE_ID.init(AtomicU8::new(0)),
            server_url: SERVER_URL.init(Watch::new_with(None)),
            status_code: STATUS_CODE.init(AtomicU8::new(0)),
            current_command: CURRENT_COMMAND.init(Watch::new_with(None)),
        }
    }
}