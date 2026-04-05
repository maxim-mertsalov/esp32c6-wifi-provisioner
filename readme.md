# ESP32-C6 Rust Wi-Fi Provisioning System
An efficient, `no_std` Rust ecosystem for configuring ESP32-C6 Wi-Fi credentials over Bluetooth Low Energy (BLE). This project provides a robust alternative to standard provisioning, optimized for bare-metal performance and low memory footprint.

## Project Vision
The goal of this project is to provide a seamless "Out-of-the-Box" experience for ESP32-C6 devices. It allows a user to:

1. Discover the device via BLE.
2. Scan for local Wi-Fi networks through the device's own radio.
3. Provision credentials (SSID/Password) and network settings.
4. Transition the device to full Wi-Fi operation while putting the BLE stack to sleep to maximize radio performance.

## Repository Structure
| Directory       | Status | Description                          |
|-----------------|--------|--------------------------------------|
| [esp32c6_stable/](esp32c6_stable/readme.md) | **Active** | Firmware source code. Uses [`esp-hal`](https://github.com/esp-rs/esp-hal) for `no_std`; [`TrouBLE`](https://github.com/embassy-rs/trouble/tree/main), `esp-radio` and `embassy_net` for BLE/Wi-Fi management |
| client_app/     | *Planned* | Desktop application to interface with the device |


## System Architecture
### 1. Provisioning Phase (BLE)

Upon boot, if no valid credentials are found (or if triggered manually), the ESP32-C6 starts a **BLE GATT Server**. The device advertises its presence, allowing the `client_app` to connect.

- **Pagination**: To handle dozens of nearby Wi-Fi networks without exceeding BLE MTU limits, the firmware uses a custom pagination protocol (16-byte SSID chunks + Network data).
- **Resource Sharing**: The radio is shared between BLE and Wi-Fi scanning.

### 2. Operational Phase (Wi-Fi)

Once credentials are received and validated:

- The BLE Server is shut down or put into a low-power state.
- The Radio is granted 100% power to the Wi-Fi stack.
- The device establishes a Station (STA) connection.


## Roadmap
- [x] Core BLE Server: Stable advertising and GATT service structure.
- [x] Wi-Fi Scanning: Logic for capturing and paginating nearby Access Points.
- [x] Credential Handling: Secure characteristic for SSID/Password transmission.
- [x] Advanced Network Config: Support for DHCP/Static IP toggles and IPv6/IPv4 selection.
- [ ] Encryption Support: UI-driven selection for WPA2, WPA3, and Enterprise protocols.
- [ ] Persistent Storage: Integration with NVS (Non-Volatile Storage) to store login data
- [ ] Client App: Cross-platform application for easy provisioning.


## Requirements & Tooling
- Language: Rust (Nightly toolchain)
- Hardware: ESP32-C6 (RISC-V)

