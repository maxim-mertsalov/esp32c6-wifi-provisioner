

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


