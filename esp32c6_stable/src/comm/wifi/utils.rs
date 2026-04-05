use core::net::{Ipv4Addr, Ipv6Addr};
use embassy_net::{ConfigV4, ConfigV6, Stack, StaticConfigV4, StaticConfigV6};
use embassy_net::IpAddress;
use embassy_net::dns::DnsQueryType;
use log::{*};
use embassy_net::{Ipv4Cidr, Ipv6Cidr};
use esp_radio::wifi::AuthMethod;
use crate::comm::wifi::models::WifiConnectionType;
use crate::errors::wifi_error::{DNSError, WifiError};

pub async fn nslookup(stack: &Stack<'static>, addr: &str) -> Result<IpAddress, WifiError> {
    let response = (*stack).dns_query(addr, DnsQueryType::A).await?;

    info!("[WIFI_task]: nslookup got response: {:?}", response);
    let addr: Option<&IpAddress> = (*response).get(0);
    if let Some(&addr) = addr {
        Ok(addr)
    }else {
        Err(WifiError::DNSError(DNSError::NotFound))
    }
}

pub async fn apply_ip_config(stack: &Stack<'static>, connection_type: WifiConnectionType) {
    match connection_type {
        WifiConnectionType::DHCP => {
            stack.set_config_v4(ConfigV4::Dhcp(Default::default()));
        }
        WifiConnectionType::Static { ip, subnet_mask, gateway } => {
            let static_config = StaticConfigV4 {
                address: Ipv4Cidr::new(Ipv4Addr::from_bits(u32::from_be_bytes(ip)), subnet_mask),
                gateway: Some(Ipv4Addr::from_bits(u32::from_be_bytes(gateway))),
                dns_servers: Default::default(),
            };
            stack.set_config_v4(ConfigV4::Static(static_config));
        }
        WifiConnectionType::DHCPv6 => {
            stack.set_config_v6(ConfigV6::default());
            todo!()
        }
        WifiConnectionType::StaticV6 { ip, prefix_length, gateway } => {
            let static_config = StaticConfigV6 {
                address: Ipv6Cidr::new(Ipv6Addr::from(ip), prefix_length),
                gateway: Some(Ipv6Addr::from(gateway)),
                dns_servers: Default::default(),
            };
            stack.set_config_v6(ConfigV6::Static(static_config));
        }
    }
}


pub fn auth_method_to_u8(auth_method: AuthMethod) -> u8 {
    match auth_method {
        AuthMethod::None => 0,
        AuthMethod::Wep => 1,
        AuthMethod::Wpa => 2,
        AuthMethod::Wpa2Personal => 3,
        AuthMethod::WpaWpa2Personal => 4,
        AuthMethod::Wpa2Enterprise => 5,
        AuthMethod::Wpa3Personal => 6,
        AuthMethod::Wpa2Wpa3Personal => 7,
        AuthMethod::WapiPersonal => 8,
        _ => 0,
    }
}

pub fn auth_method_from_u8(auth_method: u8) -> AuthMethod {
    match auth_method {
        0 => AuthMethod::None,
        1 => AuthMethod::Wep,
        2 => AuthMethod::Wpa,
        3 => AuthMethod::Wpa2Personal,
        4 => AuthMethod::WpaWpa2Personal,
        5 => AuthMethod::Wpa2Enterprise,
        6 => AuthMethod::Wpa3Personal,
        7 => AuthMethod::Wpa2Wpa3Personal,
        8 => AuthMethod::WapiPersonal,
        _ => AuthMethod::None,
    }
}