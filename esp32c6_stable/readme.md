# ESP32-C6 Firmware
This folder contains the bare-metal Rust firmware for the ESP32-C6. It implements a BLE GATT server that acts as a provisioning interface to configure Wi-Fi settings without a pre-existing network connection.

## Technical Stack
- **Runtime & Async**: `embassy` — Provides the async executor and hardware abstraction.
- **Radio Driver**: `esp-hal` — Low-level radio driver for the ESP32-C6.
- **BLE Host**: `trouble-host` — An async-first, `no_std` BLE host implementation.
- **Network Stack**: `embassy_net` — A no-std no-alloc async network stack.
- **Static Allocation**: `static_cell` — Used for safe, permanent allocation of resources.
- **Data Structures**: `heapless` — Fixed-capacity collections to ensure zero heap usage.

## BLE GATT Specification
The device exposes a single provisioning service with the following UUID:
`5f435fa5-adee-4e9a-b9a3-b812d2628906`

### 1. Wi-Fi Scanning & Results
| Characteristic         | UUID (Suffix)     | Access      | Type      | Description                                     |
|:-----------------------|:------------------|:------------|:----------|:------------------------------------------------|
| `wifi_scan_cmd`        | `...a2320fa3a6e5` | Write       | `bool`    | Set to `true` to trigger an async Wi-Fi scan.   |
| `wifi_get_status`      | `...a9e56d607e9e` | Read/Notify | `u8`      | `0`: Idle, `1`: Scanning, `2`: Connected....    |
| `wifi_get_pages_count` | `...588e8b248b95` | Read        | `u8`      | Total number of AP pages found.                 |
| `wifi_select_page`     | `...7a312c1efe54` | Write       | `u8`      | Select which page to load into the data buffer. |
| `wifi_get_page_data`   | `...369c8f646252` | Read        | `[u8; N]` | Returns SSID and RSSI for the selected page.    |

### 2. Configuration & Connection
| Characteristic             | UUID (Suffix)     | Access | Type       | Description                                         |
|:---------------------------|:------------------|:-------|:-----------|:----------------------------------------------------|
| `wifi_set_ssid_index`      | `...0020100907bc` | Write  | `u8`       | Index of the SSID from the scan list to connect to. |
| `wifi_set_password`        | `...c1e468f039f2` | Write  | `[u8; 64]` | Set the WPA2/WPA3 password.                         |
| `wifi_set_connection_type` | `...30bbfe8b1c2c` | Write  | `[u8; N]`  | Serialized `WifiConnectionType` (Static/DHCP).      |
| `wifi_connect`             | `...c3cf349ca53`  | Write  | `bool`     | Trigger connection attempt with current config.     |
| `wifi_disconnect`          | `...f1ef74a522e3` | Write  | `bool`     | Drop the current Wi-Fi connection.                  |

### 3. Testing & Diagnostics
| Characteristic     | UUID (Suffix)     | Access      | Type   | Description                                               |
|:-------------------|:------------------|:------------|:-------|:----------------------------------------------------------|
| `wifi_local_test`  | `...b597f3532d9b` | Write       | `bool` | Ping the gateway.                                         |
| `wifi_global_test` | `...e784789544f9` | Write       | `bool` | Test internet connectivity (via google.com/generate_204). |
| `status_code`      | `...ec8c3b77338e` | Read/Notify | `u8`   | System error/success codes.                               |

## Data Protocols

### SSID Pagination
To handle multiple access points within BLE MTU limits:
1. Scan for networks via `wifi_scan_cmd`.
2. Read `wifi_get_pages_count` to know the total pages count.
3. Write the desired index to `wifi_select_page`.
4. Read `wifi_get_page_data` to retrieve the SSID, signal strength and authentication method. 
5. Parse structure of page data:
```
[SSID (16 bytes)][RSSI (1 byte)][Auth Type (1 byte)]
```

### Connection Configuration
The `wifi_set_connection_type` characteristic accepts a serialized byte array representing the IP configuration. By default, it supports:
- **DHCPv4**: Dynamic address assignment.
- **StaticV4**: Pre-configured IP, Subnet Prefix and Gateway.
- **StaticV6**: Pre-configured IPv6 address, Prefix and Gateway.
- **DHCPv6**: not implemented.

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
The firmware leverages Rust's ownership model and `static_cell` to guarantee that all async tasks and hardware drivers (Radio, Timers) are allocated at startup. No heap allocator is used (`no_std`), preventing fragmentation and ensuring long-term stability for IoT provisioning.

