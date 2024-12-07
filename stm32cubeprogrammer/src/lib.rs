pub mod api_log;
pub use api_log::{LogMessageType, Verbosity};

pub mod api_types;
pub use api_types::*;

pub mod display;
pub use display::DisplayCallback;

pub mod cube_programmer;
pub use cube_programmer::{CubeProgrammer, CubeProgrammerBuilder};

pub mod error;
pub mod utility;

/// Standard base address of STM32 flash memory
pub const MCU_FLASH_BASE_ADDRESS: u32 = 0x08000000;

/// Standard base address of STM32 RAM
pub const MCU_RAM_BASE_ADDRESS: u32 = 0x20000000;