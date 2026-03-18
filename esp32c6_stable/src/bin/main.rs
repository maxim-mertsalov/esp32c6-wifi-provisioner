//! A bluetooth battery service example built using Embassy and trouBLE.

#![no_std]
#![no_main]
extern crate alloc;

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
};
use esp32c6_stable::prelude::*;

esp_bootloader_esp_idf::esp_app_desc!();


//? My code, because I got some errors and these fix them
#[unsafe(no_mangle)]
extern "C" fn __esp_radio_misc_nvs_init() -> i32 {
    0 // Returns 0 (OK)
}

#[unsafe(no_mangle)]
extern "C" fn __esp_radio_misc_nvs_deinit() {}
//? End of my code


#[esp_rtos::main]
async fn main(_s: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let board = Board::init(peripherals);

    let state = AppState::default();


    main_control_loop(board, state).await;
}



