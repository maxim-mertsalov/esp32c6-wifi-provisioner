

pub const MAX_SSID_LEN: usize = 64;
pub const MAX_PASSWORD_LEN: usize = 64;
pub const SHORT_SSID_LEN: usize = 16;
pub const SERIALIZED_SSID_LEN: usize = SHORT_SSID_LEN + 1; // SSID + RSSI
pub const MAX_SSID_PER_PAGE: usize = 5;
pub const ENTIRE_SSID_PAGE_SIZE: usize = SERIALIZED_SSID_LEN * MAX_SSID_PER_PAGE;

pub const MAX_NETWORKS_ON_DEVICE: usize = 24;

#[derive(Debug, Clone, Default)]
pub struct WifiCredentials {
    pub ssid: heapless::String<MAX_SSID_LEN>,
    pub password: heapless::String<MAX_PASSWORD_LEN>,
    // pub connection_type: WifiConnectionType
}

#[derive(Debug, Clone)]
pub enum WifiConnectionType {
    DHCP,
    Static {
        ip: [u8; 4],
        subnet_mask: [u8; 4],
        gateway: [u8; 4],
    },

    DHCPv6,
    StaticV6 {
        ip: [u8; 16],
        prefix_length: u8,
        gateway: [u8; 16],
    },
}


#[derive(Debug, Clone)]
pub struct WifiScanResult {
    pub ssid: heapless::String<MAX_SSID_LEN>,
    pub rssi: i8
}

impl WifiScanResult {
    pub fn new(ssid: heapless::String<MAX_SSID_LEN>, rssi: i8) -> Self {
        Self { ssid, rssi }
    }

    pub fn into_bytes(self) -> [u8; SERIALIZED_SSID_LEN] {
        let mut bytes = [0u8; SERIALIZED_SSID_LEN];

        let ssid_bytes = self.ssid.as_bytes();

        for i in 0..SERIALIZED_SSID_LEN {
            bytes[i] = *ssid_bytes.get(i).unwrap_or(&0u8);
        }
        bytes[SERIALIZED_SSID_LEN - 1] = self.rssi as u8;

        bytes
    }
}

#[repr(u8)]
pub enum WifiStatus {
    Idle = 0, /// Or Success in scanning
    Scanning = 1,
    Connecting = 2,
    ConnectedWithoutInternet = 3,
    Connected = 4,

    ErrorNoConnection = 251,
    ErrorConnectionFailed = 252,
    ErrorIncorrectPassword = 253,
    ErrorNoScanning = 254,
    Error = 255
}