#![no_std]

pub mod comm;
pub mod app;
pub mod board;
pub mod errors;
pub mod utils;

pub mod prelude {
    pub use crate::board::Board;
    pub use crate::app::state::AppState;

    pub use crate::app::main_control_loop;
    pub use crate::app::runner::runner_task;

    pub use crate::comm::wifi::runner::wifi_runner;
}

