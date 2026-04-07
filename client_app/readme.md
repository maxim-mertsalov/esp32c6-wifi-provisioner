# ESP32 Wi-Fi Provisioner CLI
This is a cross-platform command-line tool written in Rust for provisioning ESP32-C6 devices over BLE. It allows users to scan for Wi-Fi networks, configure connection parameters (including Static IP and IPv6), and verify connectivity directly from the terminal.

## Features
* **BLE Discovery & Connection**: Robust interaction with ESP32-C6 GATT services using `btleplug`.
* **Async Wi-Fi Scanning**: Supports paginated scanning to retrieve full lists of access points without exceeding BLE MTU limits.
* **Flexible IP Configuration**:
    * **IPv4**: DHCP or Static (IP, Subnet, Gateway).
    * **IPv6**: DHCPv6 or StaticV6 (IP, Prefix, Gateway).
* **Interactive TUI**: User-friendly menus and prompts powered by `dialoguer`.
* **Real-time Diagnostics**: Remote execution of local (gateway) and global (internet) connectivity tests.
* **Smart Security Handling**: Automatic detection of authentication methods (prompts for password only when required).

## Core Logic Flow
The application operates as a state machine that synchronizes with the ESP32 firmware via a specific status characteristic.

1.  **Command Phase**: The CLI sends a command (e.g., `WIFI_CONNECT` or `WIFI_SCAN_CMD`).
2.  **Polling Phase**: The CLI enters a `loop`, reading the `WIFI_GET_STATUS` characteristic every second.
3.  **Validation Phase**:
    * If the status matches an **Active** code (Scanning, Connecting), polling continues.
    * If the status matches a **Success** code, the loop breaks and the UI updates.
    * If the status matches an **Error** code, the CLI parses the error and displays it to the user using `anyhow`.

## User Interface & Commands
The application provides a main control menu with the following options:

### 1. Scan & Connect to Wi-Fi
* **Scanning**: Fetches networks in pages, parsing SSID, RSSI (signal strength), and Auth method.
* **Configuration**:
    * Set Password (if Auth != Open).
    * Change Connection Type (DHCP by default).
* **Static IP Input**: Includes built-in validation for IPv4 and IPv6 address formats.

### 2. Connectivity Tests
* **Local Test**: Commands the ESP32 to ping its default gateway.
* **Global Test**: Commands the ESP32 to attempt an HTTP request to `google.com/generate_204`.

### 3. Management
* **Status Check**: Reads the current `WifiStatus` from the device.
* **Disconnect**: Clears Wi-Fi credentials and drops the connection on the ESP32.


## Running the Application
Ensure you have a Bluetooth-enabled adapter and the ESP32-C6 firmware running in provisioning mode.

```bash
# Build and run the CLI
cargo run --release
```

## Dependencies
* `btleplug`: Cross-platform BLE stack.
* `tokio`: Async runtime for handling concurrent BLE events and timers.
* `dialoguer`: Command-line prompts and menus.
* `anyhow`: Flexible error handling for hardware communication.
