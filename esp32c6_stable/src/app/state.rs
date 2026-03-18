use portable_atomic::{AtomicU8, AtomicU64};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;
use static_cell::StaticCell;
use crate::comm::wifi::utils::WifiConfig;


#[derive(Copy, Clone)]
pub struct AppState {
    pub wifi_config: &'static Watch<CriticalSectionRawMutex, Option<WifiConfig>, 4>,
    pub server_url: &'static Watch<CriticalSectionRawMutex, Option<heapless::String<64>>, 4>,
}

pub enum AppStateCommand {
    SendWifiConfig(WifiConfig),
    SendServerUrl(heapless::String<64>),

    SendGetRequest,

    TestConnection,
}

impl Default for AppState {
    fn default() -> Self {
        static WIFI_CONFIG: StaticCell<Watch<CriticalSectionRawMutex, Option<WifiConfig>, 4>> = StaticCell::new();
        static SERVER_URL: StaticCell<Watch<CriticalSectionRawMutex, Option<heapless::String<64>>, 4>> = StaticCell::new();

        AppState {
            wifi_config: WIFI_CONFIG.init(Watch::new_with(None)),
            server_url: SERVER_URL.init(Watch::new_with(None)),
        }
    }
}