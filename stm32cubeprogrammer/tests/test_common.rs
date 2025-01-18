//! Common functions for the tests

use std::ffi::OsString;

use stm32cubeprogrammer::{ConnectedFusProgrammer, ConnectedProgrammer, CubeProgrammer, Verbosity};
pub enum EnvVar {
    CubeProgrammerDir,
    DownloadHexPath,
    DownloadBinPath,
    DownloadBinStartAddress,
    BleStackPath,
    BleStackStartAddress,
}

impl EnvVar {
    pub fn get(&self) -> OsString {
        std::env::var(self.as_os_string()).unwrap().into()
    }

    pub fn as_os_string(&self) -> OsString {
        let str = match self {
            EnvVar::CubeProgrammerDir => "STM32_CUBE_PROGRAMMER_DIR",
            EnvVar::DownloadHexPath => "STM32_CUBE_PROGRAMMER_DOWNLOAD_HEX_PATH",
            EnvVar::DownloadBinPath => "STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_PATH",
            EnvVar::DownloadBinStartAddress => "STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_START_ADDRESS",
            EnvVar::BleStackPath => "STM32_CUBE_PROGRAMMER_BLE_STACK_PATH",
            EnvVar::BleStackStartAddress => "STM32_CUBE_PROGRAMMER_BLE_STACK_START_ADDRESS",
        };

        str.into()
    }
}

/// Get the target directory -> Unfortunately MANIFEST_DIR does not work with cargo workspaces?
pub fn get_target_dir() -> std::path::PathBuf {
    let path = std::env::current_dir()
        .unwrap()
        .join("..")
        .join("target")
        .canonicalize()
        .unwrap();
    path
}

/// Init the CubeProgrammer
pub fn init_programmer() -> CubeProgrammer {
    dotenvy::dotenv().expect("Failed to load .env file");

    CubeProgrammer::builder()
        .cube_programmer_dir(&EnvVar::CubeProgrammerDir.get())
        .log_verbosity(Verbosity::Level3)
        .build()
        .unwrap()
}

pub fn connect_to_target<'a>(
    programmer: &'a CubeProgrammer,
    protocol: &stm32cubeprogrammer::probe::Protocol,
    connection_parameters: &stm32cubeprogrammer::probe::ConnectionParameters,
) -> ConnectedProgrammer<'a> {
    let probes = programmer.list_available_probes().unwrap();

    programmer
        .connect_to_target(&probes[0], protocol, connection_parameters)
        .unwrap()
}

pub fn connect_to_target_fus<'a>(
    programmer: &'a CubeProgrammer,
    protocol: &stm32cubeprogrammer::probe::Protocol,
) -> ConnectedFusProgrammer<'a> {
    let probes = programmer.list_available_probes().unwrap();

    programmer
        .connect_to_target_fus(&probes[0], protocol)
        .unwrap()
}
