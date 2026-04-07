

#[derive(Debug)]
pub enum BleError {
    InvalidUuid,
    ConnectionFailed,
    DisconnectionFailed,
    WriteFailed,
    ReadFailed,
    NotificationFailed,
    ServerInitializationFailed(&'static str),
    UnknownError(&'static str),
}

impl From<trouble_host::Error> for BleError {
    fn from(e: trouble_host::Error) -> Self {
        match e {
            _ => Self::UnknownError("An unknown BLE error occurred from touble-host"),
        }
    }
}
