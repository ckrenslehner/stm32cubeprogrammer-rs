//! This crate provides low level bindings to the STM32CubeProgrammer API.
//! The bindings are generated using bindgen and use the [`libloading`](https://crates.io/crates/libloading) crate to support dynamic loading of the CubeProgrammer API.
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused,
    clippy::all
)]

#[cfg(windows)]
include!("bindings_windows.rs");

#[cfg(unix)]
include!("bindings_unix.rs");

// Re-export libloading so that the user doesn't have to depend on it
pub use libloading;

/// Standard base address of STM32 flash memory
pub const FLASH_BASE_ADDRESS: u32 = 0x08000000;

/// Standard base address of STM32 RAM
pub const SRAM_BASE_ADDRESS: u32 = 0x20000000;

/// Base address of SRAM2A stm32wb5x (shared RAM containing FUS info)
pub const SRAM2A_BASE_ADDRESS_STM32WB5X: u32 = 0x20030000;

/// Base address of SRAM2A stm32wb1x (shared RAM containing FUS info)
pub const SRAM2A_BASE_ADDRESS_STM32WB1X: u32 = 0x20003000;

#[cfg(windows)]
pub const PATH_API_LIBRARY_RELATIVE: &str = "api/lib/CubeProgrammer_API.dll";

#[cfg(unix)]
pub const PATH_API_LIBRARY_RELATIVE: &str = "lib/libCubeProgrammer_API.so";

#[cfg(windows)]
pub const PATH_LOADER_DIR_RELATIVE: &str = "bin";

#[cfg(unix)]
pub const PATH_LOADER_DIR_RELATIVE: &str = "bin";
