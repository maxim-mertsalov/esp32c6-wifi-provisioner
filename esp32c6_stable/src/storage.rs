use esp_storage::FlashStorage;
use static_cell::StaticCell;

pub struct StorageRepo {
    flash: &'static FlashStorage<'static>
}

// TODO: Storage Repo
// TODO: This will store wifi credentials and other runtime settings
impl StorageRepo {}
