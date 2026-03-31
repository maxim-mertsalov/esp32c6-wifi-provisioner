use crate::comm::ble::BleGATTServer;

pub enum CharacteristicAction {
    // Wi-Fi Scan
    WifiScanCmd,
    WifiGetStatus,
    WifiGetPagesCount,
    WifiSelectPage,
    WifiGetPageData,

    // Wi-Fi connect
    WifiSetSSIDIndex,
    WifiSetPassword,
    WifiConnect,

    // Wi-Fi disconnect
    WifiDisconnect,

    // Test Wi-Fi connection
    WifiLocalTest,

    StatusCode,
}

impl BleGATTServer<'_> {
    pub fn handle_action(&self, handle: u16) -> Option<CharacteristicAction> {
        let s = &self.general_service;

        if handle == s.wifi_scan_cmd.handle {return  Some(CharacteristicAction::WifiScanCmd)}
        else if handle == s.wifi_get_status.handle {return Some(CharacteristicAction::WifiGetStatus)}
        else if handle == s.wifi_get_pages_count.handle {return Some(CharacteristicAction::WifiGetPagesCount)}
        else if handle == s.wifi_select_page.handle {return Some(CharacteristicAction::WifiSelectPage)}
        else if handle == s.status_code.handle {return Some(CharacteristicAction::StatusCode)}
        else if handle == s.wifi_get_page_data.handle {return Some(CharacteristicAction::WifiGetPageData)}
        else if handle == s.wifi_set_ssid_index.handle {return Some(CharacteristicAction::WifiSetSSIDIndex)}
        else if handle == s.wifi_set_password.handle {return Some(CharacteristicAction::WifiSetPassword)}
        else if handle == s.wifi_connect.handle {return Some(CharacteristicAction::WifiConnect)}
        else if handle == s.wifi_disconnect.handle {return Some(CharacteristicAction::WifiDisconnect)}
        else if handle == s.wifi_local_test.handle {return Some(CharacteristicAction::WifiLocalTest)}

        None
    }
}