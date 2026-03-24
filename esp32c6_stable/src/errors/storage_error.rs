use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

pub enum StorageError {
    Other(&'static str),
    OutOfBounds(&'static str),
}

impl From<esp_storage::FlashStorageError> for StorageError {
    fn from(e: esp_storage::FlashStorageError) -> Self {
        match e.kind() {
            NorFlashErrorKind::NotAligned => {
                StorageError::Other("Flash storage error: Not aligned")
            }
            NorFlashErrorKind::OutOfBounds => {
                StorageError::OutOfBounds("Flash storage error: Out of bounds")
            }
            NorFlashErrorKind::Other => {
                StorageError::Other("Flash storage error: Other")
            },
            _ => StorageError::Other("Flash storage error: Unknown"),
        }
    }
}