use log::*;
use trouble_host::{BleHostError, Controller};
use trouble_host::advertise::{AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE};
use trouble_host::gatt::GattConnection;
use trouble_host::peripheral::Peripheral;
use trouble_host::prelude::*;
use crate::comm::ble::{ble_gatt_server_uuids, BleGATTServer};


/// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
pub async fn advertise<'s, C: Controller>(
    name: &str,
    peripheral: &mut Peripheral<'static, C, DefaultPacketPool>,
    server: &'s BleGATTServer<'static>,
) -> Result<GattConnection<'static, 's, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut adv_data = [0; 31];
    let mut scan_data = [0; 31];

    let adv_len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut adv_data[..],
    )?;

    let array = ble_gatt_server_uuids::SERVICE_UUID.as_raw();

    let mut format_array = [0u8; 16];

    for i in 0..16 {
        format_array[i] = array[15 - i];
    }

    // let uuid_bytes = ble_gatt_server_uuids::SERVICE_UUID.bytes();

    let scan_len = AdStructure::encode_slice(
        &[
            AdStructure::ServiceUuids128(&[format_array]),

            AdStructure::ManufacturerSpecificData {
                company_identifier: 0x7562,
                payload: b"Meow...",
            }

            // AdStructure::Unknown {
            //     ty: 0x19,
            //     data: &[0x00, 0x02],
            // },
        ],
        &mut scan_data[..],
    )?;

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data[..adv_len],
                scan_data: &scan_data[..scan_len],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}
