use log::{info, warn};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent};
use trouble_host::{Error, PacketPool};
use crate::comm::ble::BleGATTServer;

/// Stream Events until the connection closes.
///
/// This function will handle the GATT events and process them.
/// This is how we interact with read and write requests.
pub async fn gatt_events_task<P: PacketPool>(
    server: &BleGATTServer<'_>,
    conn: &GattConnection<'_, '_, P>,
) -> Result<(), Error> {
    let schedule = server.general_service.status_code;
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        info!("[gatt] read: {:?}", event.handle());
                        if event.handle() == schedule.handle {
                            let value = server.get(&schedule);
                            info!("[gatt] Read Event to Level Characteristic: {:?}", value);
                        }
                    }
                    GattEvent::Write(event) => {
                        // info!("[gatt] Write Event to Level Characteristic: {:?}", event);
                        info!("[gatt] Write Event to Level Characteristic: {:?}", event.data());

                        // if event.handle() == service.last_used_at.handle {
                        //     info!("[gatt] Matched! Write Event to Level Characteristic: {:?}",event.data());
                        // }
                    }
                    _ => {
                        warn!("[gatt] Unhandled Gatt Event");
                    }
                };
                // This step is also performed at drop(), but writing it explicitly is necessary
                // in order to ensure reply is sent.
                match event.accept() {
                    Ok(reply) => {
                        reply.send().await
                    },
                    Err(e) => warn!("[gatt] error sending response: {:?}", e),
                };
            }
            _ => {} // ignore other Gatt Connection Events
        }
    };
    info!("[gatt] disconnected: {:?}", reason);
    Ok(())
}