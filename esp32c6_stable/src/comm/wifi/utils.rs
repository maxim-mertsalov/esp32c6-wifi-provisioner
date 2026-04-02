

pub fn create_interface(
    device: &mut esp_radio::wifi::WifiDevice,
    now: smoltcp::time::Instant
) -> smoltcp::iface::Interface {
    // Extract MAC-address from esp32 chip
    let mac_bytes = device.mac_address();
    let ethernet_addr = smoltcp::wire::EthernetAddress::from_bytes(&mac_bytes);

    // Create interface configuration
    // We use Ethernet-like device type
    let config = smoltcp::iface::Config::new(
        smoltcp::wire::HardwareAddress::Ethernet(ethernet_addr)
    );

    // Create interface
    // It uses device to send/ receive packets
    // The timestamp is used for timeouts and other time related operations in the stack
    smoltcp::iface::Interface::new(
        config,
        device,
        now
    )
}