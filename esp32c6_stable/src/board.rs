use trouble_host::prelude::*;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::peripherals::Peripherals;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::{ble, wifi};
use esp_radio::wifi::{ClientConfig, ModeConfig};
use esp_storage::FlashStorage;
use static_cell::StaticCell;
use log::*;

pub struct Board {
    // pub button: Input<static>,
    pub rgb_led: Output<'static>,

    // Wi-Fi
    pub wifi_controller: Option<wifi::WifiController<'static>>,
    pub wifi_device: Option<wifi::WifiDevice<'static>>,

    // Bluetooth
    pub ble_controller: ExternalController<ble::controller::BleConnector<'static>, 1>,

    // Flash Storage
    pub flash_storage: &'static FlashStorage<'static>,

    // Random number generator module
    pub rng: Rng,
}

impl Board {
    pub fn init(peripherals: Peripherals) -> Self {
        let timg0 = TimerGroup::new(peripherals.TIMG0);
        let sw_interrupt = esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

        // Start the RTOS scheduler with the timer and software interrupt
        esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
        info!("Embassy initialized!");

        // Random number generator
        let rng = Rng::new();

        // ESP radio initialization
        static RADIO: StaticCell<esp_radio::Controller<'static>> = StaticCell::new();
        let radio = RADIO.init(esp_radio::init().unwrap());


        // Bluetooth controller
        let bluetooth = peripherals.BT;
        let connector = ble::controller::BleConnector::new(radio, bluetooth, Default::default()).unwrap();
        let controller: ExternalController<_, 1> = ExternalController::new(connector);


        // Wifi
        let (mut wifi_controller, wifi_interfaces) = wifi::new(
            radio,
            peripherals.WIFI,
            wifi::Config::default()
        ).unwrap();
        let device = wifi_interfaces.sta;
        wifi_controller.set_config(&ModeConfig::Client(ClientConfig::default()))
            .expect("Failed to set Wi-Fi mode");


        // Flash Storage
        let flash_storage = FlashStorage::new(peripherals.FLASH);
        static FLASH_STORAGE: StaticCell<FlashStorage> = StaticCell::new();
        let flash_storage_ref = FLASH_STORAGE.init(flash_storage);


        // RGB LED
        let rgb_led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

        Self {
            rgb_led,
            wifi_controller: Some(wifi_controller),
            wifi_device: Some(device),
            ble_controller: controller,
            flash_storage: flash_storage_ref,
            rng,
        }


    }
}