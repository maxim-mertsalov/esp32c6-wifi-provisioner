use std::fmt::{Display, Formatter};
use crate::utils::auth_method_from_u8;

pub const SHORT_SSID_LEN: usize = 16;
pub const SERIALIZED_SSID_LEN: usize = SHORT_SSID_LEN + 1 + 1; // SSID + RSSI + Security Type
pub const MAX_SSID_PER_PAGE: usize = 5;
pub const ENTIRE_SSID_PAGE_SIZE: usize = SERIALIZED_SSID_LEN * MAX_SSID_PER_PAGE;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AuthMethod {
    None,
    Wep,
    Wpa,
    Wpa2Personal,
    WpaWpa2Personal,
    Wpa2Enterprise,
    Wpa3Personal,
    Wpa2Wpa3Personal,
    WapiPersonal,
}

#[derive(Debug, Clone)]
pub struct WifiScanResult {
    pub ssid: String,
    pub rssi: i8,
    pub auth_method: AuthMethod
}

impl WifiScanResult {
    pub fn from_bytes(data: &[u8]) -> Vec<WifiScanResult> {
        let mut result: Vec<WifiScanResult> = Vec::new();

        let mut i = 0;
        loop {
            if i >= data.len() { break; }
            let ssid = String::from_utf8_lossy(&data[i..i + SHORT_SSID_LEN]).trim_end_matches(&['\0', '\n']).to_string();
            if ssid.is_empty() { break; }

            let rssi = data[i + SERIALIZED_SSID_LEN - 2] as i8;
            let auth_method_raw = data[i + SERIALIZED_SSID_LEN - 1];
            let auth_method = auth_method_from_u8(auth_method_raw);

            let scan_result = WifiScanResult {
                ssid,
                rssi,
                auth_method
            };
            result.push(scan_result);
            i += SERIALIZED_SSID_LEN;
        }


        result
    }
}



#[derive(Debug, Clone, Default)]
pub struct WifiCredentials {
    pub network_index: usize,
    pub password: String,
    pub connection_type: WifiConnectionType
}

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

impl Display for WifiConnectionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WifiConnectionType::DHCP => write!(f, "DHCP"),
            WifiConnectionType::Static { ip, subnet_mask, gateway } => write!(f, "Static (IP: {}.{}.{}.{}, Prefix: {}, Gateway: {}.{}.{}.{})", ip[0], ip[1], ip[2], ip[3], subnet_mask, gateway[0], gateway[1], gateway[2], gateway[3]),
            WifiConnectionType::DHCPv6 => write!(f, "DHCPv6"),
            WifiConnectionType::StaticV6 { ip, prefix_length, gateway } => write!(f, "StaticV6 (IP: {:x?}, Prefix Length: {}, Gateway: {:x?})", ip, prefix_length, gateway),
        }
    }
}

impl WifiConnectionType {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            WifiConnectionType::DHCP => {
                bytes.push(0);
            }
            WifiConnectionType::Static { ip, subnet_mask, gateway } => {
                bytes.push(1);
                bytes.extend_from_slice(ip);
                bytes.push(*subnet_mask);
                bytes.extend_from_slice(gateway);
            }
            WifiConnectionType::DHCPv6 => {
                bytes.push(2);
            }
            WifiConnectionType::StaticV6 { ip, prefix_length, gateway } => {
                bytes.push(3);
                bytes.extend_from_slice(ip);
                bytes.push(*prefix_length);
                bytes.extend_from_slice(gateway);
            }
        }

        bytes
    }
}
