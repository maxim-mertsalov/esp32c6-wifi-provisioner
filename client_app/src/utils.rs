use btleplug::platform::{Peripheral as BlePeripheral};
use anyhow::Result;
use btleplug::api::Peripheral;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use crate::models::AuthMethod;

pub async fn select_peripheral(peripherals: Vec<BlePeripheral>) -> Result<BlePeripheral> {
    let mut names = Vec::new();
    for p in &peripherals {
        let props = p.properties().await?.unwrap_or_default();
        let name = props.local_name.unwrap_or_else(|| "Unknown Device".to_string());
        names.push(format!("{} [{}]", name, p.address()));
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Выберите устройство")
        .items(&names)
        .default(0)
        .interact()?;

    Ok(peripherals.into_iter().nth(selection).unwrap())
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


pub fn format_rssi_level(rssi: i8) -> &'static str {
    match rssi {
        r if r >= -50 => "▂▄▆█",
        r if r >= -60 => "▂▄▆ ",
        r if r >= -70 => "▂▄  ",
        r if r >= -80 => "▂   ",
        _ => "    ",
    }
}