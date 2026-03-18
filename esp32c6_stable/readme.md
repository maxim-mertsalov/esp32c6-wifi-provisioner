# ESP32-C6 Firmware
This folder contains the bare-metal Rust firmware for the ESP32-C6. It implements a BLE GATT server that acts as a provisioning interface to configure Wi-Fi settings without a pre-existing network connection

## Technical Stack
- **Runtime & Async**: `Embassy` — Provides the async executor and hardware abstraction.
Radio Driver: `esp-radio` — Low-level radio driver for the ESP32-C6.
- **BLE Host**: `trouble-host` — An async-first, `no_std` BLE host implementation.
- **Network Stack**: `smoltcp` — A standalone, event-driven TCP/IP stack for bare-metal containers.
- **Static Allocation**: `static_cell` — Used for safe, permanent allocation of long-lived async resources.
- **Data Structures**: `heapless` — Fixed-capacity collections to ensure zero heap usage.

## BLE GATT Specification
The device exposes a specialized provisioning service

Service UUID: `ble_gatt_server_uuids::SERVICE_UUID`

| Characteristic | UUID | Access | Type | Description |
| :--- | :--- | :--- | :--- | :--- |
| `wifi_scan_cmd` | `WIFI_SCAN_CMD` | Write | `bool` | Set to `true` to trigger an async Wi-Fi scan. |
| `wifi_get_status` | `WIFI_GET_STATUS` | Read/Notify | `u8` | `0`: Idle, `1`: Scanning, `2`: Connected. |
| `wifi_get_list_count`| `WIFI_GET_LIST_COUNT`| Read/Notify | `u8` | Number of APs found in the results buffer. |
| `wifi_select_page` | `WIFI_SELECT_PAGE` | Write | `u8` | Index of the network to load into the data buffer. |
| `wifi_get_page_data` | `WIFI_GET_PAGE_DATA` | Read | `[u8; PAGE_SIZE]` | Returns 16 bytes (SSID) + 1 byte (RSSI) per page |
| `status_code` | `STATUS_CODE` | Read/Notify | `u8` | Error/Success codes (e.g., Auth failure, timeout). |


## Async Architecture
By using **Embassy** and **trouble-host**, the firmware handles concurrent tasks efficiently:
1.  **BLE Task:** Manages the GATT server and handles incoming connection requests.
2.  **Wifi/Radio Task:** Manages the `esp-radio` state and `smoltcp` interface.
3.  **Command Channel:** BLE writes trigger signals that the Wi-Fi task consumes to start scanning or connecting.


## Data Protocol: SSID Pagination
To maintain a small memory footprint and stay within BLE MTU limits:
- The `wifi_get_page_data` buffer returns an array of pages as slice.
- Clients iterate through the results by updating the index via `wifi_select_page`.
- SSIDs exceeding 16 characters are currently truncated for the summary view.


## Build & Flash

### Prerequisites
- Rust Nightly (required for some async features).
- `espflash`.

### Execution
```bash
# Build and flash the project
cargo build --release
```

## State & Safety
This project uses `static_cell` to ensure that drivers and network buffers live for the entire duration of the program without needing a heap allocator. This guarantees **memory safety** and **predictable resource usage** at compile time.