#[cfg(test)]
use log::{debug, info, warn};
use stm32cubeprogrammer::{CubeProgrammer, ResetMode, Verbosity};

/// Environment variable name for the path to the STM32CubeProgrammer directory
/// Needs to be the root path of the STM32CubeProgrammer installatios
/// Example: C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer
const ENV_CUBE_PROGRAMMER_DIR: &str = "STM32_CUBE_PROGRAMMER_DIR";

/// The path to a .hex file for downloading to the target
const ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_HEX_PATH: &str = "STM32_CUBE_PROGRAMMER_DOWNLOAD_HEX_PATH";

/// The path to a .bin file for downloading to the target
const ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_PATH: &str = "STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_PATH";

/// The start address for the .bin file download
const ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_START_ADDRESS: &str =
    "STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_START_ADDRESS";

/// The path to the BLE stack binary file
const ENV_STM32_CUBE_PROGRAMMER_BLE_STACK_PATH: &str = "STM32_CUBE_PROGRAMMER_BLE_STACK_PATH";

/// The start address for the BLE stack binary file
const STM32_CUBE_PROGRAMMER_BLE_STACK_START_ADDRESS: &str =
    "STM32_CUBE_PROGRAMMER_BLE_STACK_START_ADDRESS";

/// Get the path to the STM32CubeProgrammer directory from the environment file
fn get_path_from_env_file(env_key: &str) -> std::path::PathBuf {
    dotenvy::dotenv().unwrap();
    std::env::var(env_key)
        .expect(&format!("{} not found in .env file", env_key))
        .into()
}

/// Decodes a u32 from an environment file -> The env value is a hex string
fn get_address_from_env_file(env_key: &str) -> u32 {
    dotenvy::dotenv().unwrap();
    let hex_string = std::env::var(env_key).expect(&format!("{} not found in .env file", env_key));

    let address = if hex_string.starts_with("0x") || hex_string.starts_with("0X") {
        u32::from_str_radix(&hex_string[2..], 16).unwrap()
    } else {
        u32::from_str_radix(&hex_string, 16).unwrap()
    };

    debug!("Address from env key {}: 0x{:X}", env_key, address);
    address
}

/// Get the target directory -> Unfortunately MANIFEST_DIR does not work with cargo workspaces?
fn get_target_dir() -> std::path::PathBuf {
    let path = std::env::current_dir().unwrap();

    debug!("Current dir: {:?}", path);

    let path = path.join("..").join("target").canonicalize().unwrap();

    debug!("Target dir: {:?}", path);
    path
}

#[test_log::test]
fn discover_st_links() {
    dotenvy::dotenv().unwrap();

    let programmer = CubeProgrammer::builder()
        .cube_programmer_dir(get_path_from_env_file(ENV_CUBE_PROGRAMMER_DIR))
        .build()
        .unwrap();

    let probes = programmer.list_connected_st_link_probes();

    for probe in probes {
        info!("Found ST-Link probe: {}", probe);
    }
}

#[test_log::test]
fn connect_to_target() {
    dotenvy::dotenv().unwrap();

    let programmer = CubeProgrammer::builder()
        .cube_programmer_dir(get_path_from_env_file(ENV_CUBE_PROGRAMMER_DIR))
        .build()
        .unwrap();

    let probes = programmer.list_connected_st_link_probes();

    if !probes.is_empty() {
        info!("Found {} ST-Link probes - Trying to connect", probes.len());
        info!("Connecting to target via probe: {}", probes[0]);

        let connected_programmer = programmer.connect_to_target(&probes[0]).unwrap();

        let target_information = connected_programmer
            .get_general_device_information()
            .unwrap();

        info!("Connected to target: {}", target_information);

        info!("Connected to target. Disconnecting...");
        connected_programmer.disconnect();
    } else {
        info!("No ST-Link probes found");
    }
}

#[test_log::test]
fn download_hex_file() {
    dotenvy::dotenv().unwrap();

    let programmer = CubeProgrammer::builder()
        .cube_programmer_dir(get_path_from_env_file(ENV_CUBE_PROGRAMMER_DIR))
        .build()
        .unwrap();

    let probes = programmer.list_connected_st_link_probes();

    if !probes.is_empty() {
        info!("Found {} ST-Link probes - Trying to connect", probes.len());
        info!("Connecting to target via probe: {}", probes[0]);

        let connected_programmer = programmer.connect_to_target(&probes[0]).unwrap();

        let target_information = connected_programmer
            .get_general_device_information()
            .unwrap();

        info!("Connected to target: {}", target_information);

        let hex_file = get_path_from_env_file(ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_HEX_PATH);
        info!("Downloading hex file: {:?}", hex_file);

        connected_programmer
            .download_hex_file(hex_file, false, true)
            .unwrap();

        info!("Connected to target. Disconnecting...");

        connected_programmer
            .reset_target(ResetMode::HardwareReset)
            .unwrap();
        connected_programmer.disconnect();
    } else {
        info!("No ST-Link probes found");
    }
}

#[test_log::test]
fn download_bin_file() {
    dotenvy::dotenv().unwrap();

    let programmer = CubeProgrammer::builder()
        .cube_programmer_dir(get_path_from_env_file(ENV_CUBE_PROGRAMMER_DIR))
        .build()
        .unwrap();

    let probes = programmer.list_connected_st_link_probes();

    if !probes.is_empty() {
        info!("Found {} ST-Link probes - Trying to connect", probes.len());
        info!("Connecting to target via probe: {}", probes[0]);

        let connected_programmer = programmer.connect_to_target(&probes[0]).unwrap();

        let target_information = connected_programmer
            .get_general_device_information()
            .unwrap();

        info!("Connected to target: {}", target_information);

        let hex_file = get_path_from_env_file(ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_PATH);
        info!("Downloading bin file: {:?}", hex_file);

        connected_programmer
            .download_bin_file(
                hex_file,
                get_address_from_env_file(ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_START_ADDRESS),
                false,
                true,
            )
            .unwrap();

        info!("Connected to target. Disconnecting...");

        connected_programmer
            .reset_target(ResetMode::HardwareReset)
            .unwrap();
        connected_programmer.disconnect();
    } else {
        info!("No ST-Link probes found");
    }
}

#[test_log::test]
fn upgrade_ble_stack() {
    dotenvy::dotenv().unwrap();

    let programmer = CubeProgrammer::builder()
        .cube_programmer_dir(get_path_from_env_file(ENV_CUBE_PROGRAMMER_DIR))
        .build()
        .unwrap();

    let probes = programmer.list_connected_st_link_probes();

    if !probes.is_empty() {
        info!("Found {} ST-Link probes - Trying to connect", probes.len());
        info!("Connecting to target via probe: {}", probes[0]);

        let connected_programmer = programmer.connect_to_target(&probes[0]).unwrap();

        let target_information = connected_programmer
            .get_general_device_information()
            .unwrap();

        info!("Connected to target: {}", target_information);

        let ble_stack_binary = get_path_from_env_file(ENV_STM32_CUBE_PROGRAMMER_BLE_STACK_PATH);
        info!("Downloading ble stack file: {:?}", ble_stack_binary);

        connected_programmer
            .update_ble_stack(
                ble_stack_binary,
                get_address_from_env_file(STM32_CUBE_PROGRAMMER_BLE_STACK_START_ADDRESS),
                false,
                true,
                true,
            )
            .unwrap();

        info!("Connected to target. Disconnecting...");

        connected_programmer
            .reset_target(ResetMode::HardwareReset)
            .unwrap();
        connected_programmer.disconnect();
    } else {
        info!("No ST-Link probes found");
    }
}

/// Test showing how to register a custom display handler
/// This can be used in e.g. a CLI or GUI application to show the progress of the operations
#[test_log::test]
fn register_display_handler() {
    use std::sync::Arc;

    /// Custom display handler
    struct MyDisplayHandler;

    impl stm32cubeprogrammer::DisplayCallback for MyDisplayHandler {
        fn init_progressbar(&self) {
            warn!("MyDisplayHandler - Init progress bar");
        }

        fn log_message(&self, message_type: stm32cubeprogrammer::LogMessageType, message: &str) {
            info!(
                "MyDisplayHandler - Log message: {:?} - {}",
                message_type, message
            );
        }

        fn update_progressbar(&self, current_number: u64, total_number: u64) {
            warn!(
                "MyDisplayHandler - Update progress bar: {}/{}",
                current_number, total_number
            );
        }
    }

    dotenvy::dotenv().unwrap();

    let programmer = CubeProgrammer::builder()
        .cube_programmer_dir(get_path_from_env_file(ENV_CUBE_PROGRAMMER_DIR))
        .log_verbosity(Verbosity::Level2)
        .display_callback(Arc::new(MyDisplayHandler))
        .build()
        .unwrap();

    let probes = programmer.list_connected_st_link_probes();

    if !probes.is_empty() {
        info!("Found {} ST-Link probes - Trying to connect", probes.len());
        info!("Connecting to target via probe: {}", probes[0]);

        let connected_programmer = programmer.connect_to_target(&probes[0]).unwrap();

        let target_information = connected_programmer
            .get_general_device_information()
            .unwrap();

        info!("Connected to target: {}", target_information);

        // Read the memory and store it to the target dir
        connected_programmer
            .save_memory_file(
                get_target_dir().join("memory.bin"),
                get_address_from_env_file(ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_START_ADDRESS),
                1024 * 400,
            )
            .unwrap();

        connected_programmer.disconnect();
    } else {
        info!("No ST-Link probes found");
    }
}
