use uuid::{Uuid, uuid};

/// This is list of constants from BLE GATT server definition.
/// They are used both in server and client code, so they are defined in separate module.

pub const SERVICE_UUID: Uuid = uuid!("5f435fa5-adee-4e9a-b9a3-b812d2628906");

pub const WIFI_SCAN_CMD: Uuid = uuid!("d88a7d46-9313-4240-915a-a2320fa3a6e5");
pub const WIFI_GET_STATUS: Uuid = uuid!("878b58f2-4d44-4178-ae8f-a9e56d607e9e");

pub const WIFI_GET_PAGES_COUNT: Uuid = uuid!("0ce70db8-be92-4160-a2d3-588e8b248b95");
pub const WIFI_SELECT_PAGE: Uuid = uuid!("6839a19d-8a4b-4691-89cc-7a312c1efe54");
pub const WIFI_GET_PAGE_DATA: Uuid = uuid!("9c0c07d7-0435-4a9d-b999-369c8f646252");

pub const WIFI_SET_SSID_INDEX: Uuid = uuid!("824f9460-5d76-4498-a549-0020100907bc");
pub const WIFI_SET_PASSWORD: Uuid = uuid!("273d7528-c072-4fe6-b29b-c1e468f039f2");
pub const WIFI_SET_CONNECTION_TYPE: Uuid = uuid!("25422a9b-558d-49f1-8db9-30bbfe8b1c2c");
pub const WIFI_CONNECT: Uuid = uuid!("2c1f2d97-5c53-435b-940c-c36cf349ca53");

pub const WIFI_DISCONNECT: Uuid = uuid!("61cd3e5f-0a78-4318-9891-f1ef74a522e3");

pub const WIFI_LOCAL_TEST: Uuid = uuid!("54477984-44ea-4dbb-8740-b597f3532d9b");
pub const WIFI_GLOBAL_TEST: Uuid = uuid!("24b71d12-4637-4bd1-b408-e784789544f9");

pub const STATUS_CODE: Uuid = uuid!("7df744c9-3a9b-4df6-80f3-ec8c3b77338e");