

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
    pub connection_type: WifiConnectionType
}

/// The biggest input is IPv6: 1 byte type + 16 bytes IPv6 + 1 byte prefix_length + 16 bytes of gateway
pub const MAX_WIFI_CONNECTION_TYPE_SIZE: usize = 1 + 16 + 1 + 16;

/// Serialised data is sequentially represented in bytes, example:
/// ```rust,no_run
/// use crate::esp32c6_stable::comm::wifi::models::{WifiConnectionType, MAX_WIFI_CONNECTION_TYPE_SIZE};
///
/// let connection_type = WifiConnectionType::Static {
///     ip: [192, 168, 1, 100],
///     subnet_mask: 24,
///     gateway: [192, 168, 1, 1],
/// };
///
/// let mut bytes = [0u8; MAX_WIFI_CONNECTION_TYPE_SIZE];
/// let serialised_part = [1, 192, 168, 1, 100, 24, 192, 168, 1, 1];
/// bytes[..serialised_part.len()].copy_from_slice(&serialised_part);
///
/// let deserialized = WifiConnectionType::from_bytes(bytes);
/// assert_eq!(connection_type, deserialized);
/// ```
///
#[derive(Debug, Clone, Default)]
pub enum WifiConnectionType {
    #[default]
    DHCP,
    Static {
        ip: [u8; 4],
        subnet_mask: u8,
        gateway: [u8; 4],
    },

    DHCPv6,
    StaticV6 {
        ip: [u8; 16],
        prefix_length: u8,
        gateway: [u8; 16],
    },
}

impl WifiConnectionType {
    pub fn from_bytes(bytes: [u8; MAX_WIFI_CONNECTION_TYPE_SIZE]) -> Self {
        let connection_type = bytes[0];
        match connection_type {
            0 => Self::DHCP,
            1 => Self::Static {
                ip: bytes[1..5].try_into().unwrap_or([0u8; 4]),
                subnet_mask: bytes[5].min(32).max(1),
                gateway: bytes[6..10].try_into().unwrap_or([0u8; 4]),
            },
            2 => Self::DHCPv6,
            3 => Self::StaticV6 {
                ip: bytes[1..17].try_into().unwrap_or([0u8; 16]),
                prefix_length: bytes[17].min(128).max(1),
                gateway: bytes[18..34].try_into().unwrap_or([0u8; 16]),
            },
            _ => Self::DHCP,
        }
    }
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
    ConnectedWithInterner = 5,

    ErrorNoInternet = 250,
    ErrorNoConnection = 251,
    ErrorConnectionFailed = 252,
    ErrorIncorrectPassword = 253,
    ErrorNoScanning = 254,
    Error = 255
}