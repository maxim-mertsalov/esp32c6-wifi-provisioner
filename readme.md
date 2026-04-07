# ESP32-C6 Rust Wi-Fi Provisioning System
An efficient, `no_std` Rust ecosystem for configuring ESP32-C6 Wi-Fi credentials over Bluetooth Low Energy (BLE). This project provides a robust alternative to standard provisioning, optimized for bare-metal performance and a zero-heap memory footprint.

## Project Vision
The goal of this project is to provide a seamless "Out-of-the-Box" experience for ESP32-C6 devices using a modern async-first approach.

1.  **Discovery**: Identify the device via BLE.
2.  **Scan**: Retrieve local Wi-Fi networks using the device's radio.
3.  **Provision**: Configure credentials and advanced network settings (IPv4/IPv6, Static/DHCP).
4.  **Validate**: Perform end-to-end connectivity tests (Local/Global) before switching to full Wi-Fi operation.

## Repository Structure
| Directory                                       | Status     | Description                                                                                                      |
|:------------------------------------------------|:-----------|:-----------------------------------------------------------------------------------------------------------------|
| **[esp32c6_stable/](esp32c6_stable/readme.md)** | **Active** | Firmware source code. Uses `esp-hal` for hardware abstraction and `embassy_net` for a native async network stack.|
| **[client_app/](client_app/readme.md)**         | **Active** | Interactive CLI tool for desktop (Linux/macOS/Windows) to interface with the device.                             |

## System Architecture
### 1. Provisioning Phase (BLE)
Upon boot (or manual trigger), the ESP32-C6 starts a **BLE GATT Server** via the `trouble-host` stack.

- **Async Execution**: The system leverages `embassy` to manage BLE, Wi-Fi, and application logic concurrently without an OS.
- **Pagination**: Handles large lists of nearby APs via a custom pagination protocol (90-byte chunks containing 5 networks each).
- **Status Reporting**: A dedicated status characteristic provides real-time feedback on the internal state (Scanning, Connecting, Testing).

### 2. Operational Phase (Wi-Fi)
Once credentials are confirmed and validated:

- The device establishes a Station (STA) connection using `embassy_net`.
- Supports advanced IP configurations: **Static IPv4/IPv6** and **DHCP**.
- The BLE stack can be put into a low-power state or shut down to prioritize radio performance.

## Roadmap

- [x] **Core BLE Server**: Stable advertising and GATT service structure.
- [x] **Wi-Fi Scanning**: Logic for capturing and paginating nearby Access Points.
- [x] **Advanced Network Config**: Support for DHCP/Static IP toggles and IPv4/IPv6 selection.
- [x] **Connectivity Testing**: Built-in Local (Gateway) and Global (Internet) verification.
- [x] **Client App**: Interactive CLI for easy provisioning.
- [x] **Encryption Support**: WPA2/WPA3 support (Note: WPA-Enterprise is currently out of scope).
- [ ] **Persistent Storage**: Integration with NVS (Non-Volatile Storage) for credential persistence.

## Requirements & Tooling

- **Language**: Rust (Nightly toolchain)
- **Hardware**: ESP32-C6 (RISC-V)
- **Tools**: `espflash` for firmware deployment.