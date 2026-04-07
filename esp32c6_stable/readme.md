# ESP32-C6 Firmware
This folder contains the bare-metal Rust firmware for the ESP32-C6. It implements a BLE GATT server acting as a provisioning interface to configure Wi-Fi settings (IPv4/IPv6, Static/DHCP) without a pre-existing network connection.

## System Architecture
The firmware utilizes the `embassy` async executor to manage three primary concurrent tasks. Communication between tasks is handled via async channels and shared state.

- `main_control_loop`: Manages the BLE stack lifecycle and hardware triggers. It toggles Bluetooth visibility based on external events (e.g., button presses).
- `runner_task`: The central **Command Dispatcher**. It receives commands from the BLE interface, manages application logic (like SSID pagination), and routes requests to the Wi-Fi stack.
- `wifi_runner`: Owns the `esp-wifi` controller and the `embassy_net` stack. It executes hardware-level operations: scanning, connecting, and performing connectivity tests.

## Technical Stack
- **Runtime & Async**: `embassy` — Provides the async executor and hardware abstraction.
- **Network Stack**: `embassy_net` — No-std, no-alloc async network stack (TCP/IP, DHCP, ICMP).
- **BLE Host**: `trouble-host` — An async-first, `no_std` BLE host implementation.
- **Hardware**: `esp-hal` — Low-level drivers for ESP32-C6
- **Static Allocation**: `static_cell` — Used for safe, permanent allocation of resources.
- **Memory Management**: `static_cell` & `heapless` — Guaranteed zero heap usage after initialization

## BLE GATT Specification
All UUIDs can be found in the `src/comm/ble/mod.rs` in `ble_gatt_server_uuids` module. 
The device exposes a single provisioning service with the following UUID:
`5f435fa5-adee-4e9a-b9a3-b812d2628906`

### 1. Wi-Fi Scanning & Results
| Characteristic         | UUID (Suffix)                          | Access      | Type       | Description                                                                                    |
|:-----------------------|:---------------------------------------|:------------|:-----------|:-----------------------------------------------------------------------------------------------|
| `wifi_scan_cmd`        | `d88a7d46-9313-4240-915a-a2320fa3a6e5` | Write       | `bool`     | Set to `true` to trigger an async Wi-Fi scan.                                                  |
| `wifi_get_status`      | `878b58f2-4d44-4178-ae8f-a9e56d607e9e` | Read/Notify | `u8`       | `0`: Idle, `1`: Scanning... Other codes can be found in `src/comm/wifi/models.rs` `WifiStatus` |
| `wifi_get_pages_count` | `0ce70db8-be92-4160-a2d3-588e8b248b95` | Read        | `u8`       | Total number of AP pages found.                                                                |
| `wifi_select_page`     | `6839a19d-8a4b-4691-89cc-7a312c1efe54` | Write       | `u8`       | Select which page to load into the data buffer.                                                |
| `wifi_get_page_data`   | `9c0c07d7-0435-4a9d-b999-369c8f646252` | Read        | `[u8; 90]` | Returns SSID and other data of network for the selected page.                                  |

### 2. Configuration & Connection
| Characteristic             | UUID (Suffix)                          | Access | Type       | Description                                         |
|:---------------------------|:---------------------------------------|:-------|:-----------|:----------------------------------------------------|
| `wifi_set_ssid_index`      | `824f9460-5d76-4498-a549-0020100907bc` | Write  | `u8`       | Index of the SSID from the scan list to connect to. |
| `wifi_set_password`        | `273d7528-c072-4fe6-b29b-c1e468f039f2` | Write  | `[u8; 64]` | Set the WPA2/WPA3 password.                         |
| `wifi_set_connection_type` | `25422a9b-558d-49f1-8db9-30bbfe8b1c2c` | Write  | `[u8; 34]` | Serialized `WifiConnectionType` (Static/DHCP).      |
| `wifi_connect`             | `2c1f2d97-5c53-435b-940c-c36cf349ca53` | Write  | `bool`     | Trigger connection attempt with current config.     |
| `wifi_disconnect`          | `61cd3e5f-0a78-4318-9891-f1ef74a522e3` | Write  | `bool`     | Drop the current Wi-Fi connection.                  |

### 3. Testing & Diagnostics
| Characteristic     | UUID (Suffix)                          | Access      | Type   | Description                                |
|:-------------------|:---------------------------------------|:------------|:-------|:-------------------------------------------|
| `wifi_local_test`  | `54477984-44ea-4dbb-8740-b597f3532d9b` | Write       | `bool` | Ping the gateway.                          |
| `wifi_global_test` | `24b71d12-4637-4bd1-b408-e784789544f9` | Write       | `bool` | Test internet via google.com/generate_204. |
| `status_code`      | `7df744c9-3a9b-4df6-80f3-ec8c3b77338e` | Read/Notify | `u8`   | System error/success codes.                |

## Data Protocols

### Status Codes (`WifiStatus`)
Clients should monitor `wifi_get_status` to update UI state

| Code        | Name                                                                                               | Category                          |
|:------------|:---------------------------------------------------------------------------------------------------|:----------------------------------|
| **0**       | `Idle`                                                                                             | System is waiting.                |
| **1-50**    | `Scanning`, `Connecting`, `Disconnecting`, `SendingLocalTest`, `SendingGlobalTest`                 | **Processing**: Task in progress. |
| **51-100**  | `ScannedSuccessfully`, `Connected`, `Disconnected`, `LocalTestSuccess`, `GlobalTestSuccess`        | **Success**: Operation completed. |
| **201-250** | `ErrorWhileScanning`, `ErrorWhileConnecting`, `ErrorWithLocalTest`, `ErrorNoScannedNetworks`, etc. | **Error**: Operation failed.      |
| **255**     | `Error`                                                                                            | Global system error.              |


### SSID Pagination
Data buffer (wifi_get_page_data) structure:
- **Chunk Size**: 18 bytes per entry.
- **Entry Layout**: `[<SSID (16 bytes)>, <RSSI (1 byte)> <Auth Type (1 byte)>]`.
- **Buffer Total**: 5 entries per page = 90 bytes.

### Connection Type Serialization
The `wifi_set_connection_type` characteristic accepts a serialized byte array (max 34 bytes):
- **DHCP (v4)**: `[0]`
- **Static (v4)**: `[1, <IP (4 bytes)>, <SubnetMask (1 byte)>, <Gateway (4 bytes)>]`
- **DHCP (v6)**: `[2]`
- **Static (v6)**: `<[3, <IP (16 bytes)>, <Prefix (1 byte)>, <Gateway (16 bytes)>]`

## Build & Flash

### Prerequisites
- Rust Nightly
- `espflash`

### Execution
```bash
# Build and flash to an ESP32-C6
cargo run --release
```

## State & Safety
The firmware leverages Rust's ownership model and `static_cell` to ensure all async tasks, network buffers, and hardware drivers are allocated at startup. This `no_std` approach prevents heap fragmentation and ensures long-term stability for IoT provisioning.

