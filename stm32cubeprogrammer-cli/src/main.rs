mod display_handler;
use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use display_handler::DisplayHandler;
use std::{env, sync::Mutex};
use stm32cubeprogrammer::{fus, probe};

#[derive(Debug, Clone)]
struct HexAddress(u32);

impl HexAddress {
    fn from_cli_input(hex: &str) -> Result<Self, String> {
        let address = if hex.starts_with("0x") || hex.starts_with("0X") {
            &hex[2..]
        } else {
            hex
        };

        u32::from_str_radix(address, 16)
            .map_err(|x| format!("Failed to parse {} as u32 number: {}", address, x))
            .map(Self)
    }
}

/// Binary file information for downloading
#[derive(Debug, Clone)]
struct BinFileInfo {
    /// The start address where the binary file should be downloaded
    address: HexAddress,
    /// The path to the binary file
    path: std::path::PathBuf,
}

impl std::fmt::Display for BinFileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display the address in hex format and the path
        write!(f, "{:#x}, {:?}", self.address.0, self.path)
    }
}

fn bin_file_info_from_cli_input(s: &str) -> Result<BinFileInfo, String> {
    let parts: Vec<&str> = s.split(',').collect();

    if parts.len() != 2 {
        return Err(format!("Invalid format: {}", s));
    }

    let address = HexAddress::from_cli_input(parts[0])?;
    let path = std::path::PathBuf::from(parts[1]);

    Ok(BinFileInfo { address, path })
}

fn version_from_cli_input(s: &str) -> Result<fus::Version, String> {
    fus::Version::try_from(s).map_err(|x| x.to_string())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// The verbosity level
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    /// Serial of the st-link probe
    #[arg(long)]
    serial: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ReadProtection {
    Enable,
    Disable,
}

#[derive(Subcommand)]
enum Commands {
    MassErase,
    Reset,
    ReatProtection {
        enable: ReadProtection,
    },
    DownloadBin {
        /// Binary file info. Specified as `hex_address,path/to/file.bin`
        ///
        /// Example: `0x08000000,my_bin.bin`
        #[arg(value_parser=bin_file_info_from_cli_input)]
        bin_file: BinFileInfo,

        /// Whether to skip the erase operation before downloading the hex file
        #[arg(long, default_value_t = false)]
        skip_erase: bool,

        /// Whether to skip the verify operation after downloading the hex file
        #[arg(long, default_value_t = false)]
        skip_verify: bool,
    },
    DownloadHex {
        /// Path to the hex file to download
        hex_file: std::path::PathBuf,

        /// Whether to skip the erase operation before downloading the hex file
        #[arg(long, default_value_t = false)]
        skip_erase: bool,

        /// Whether to skip the verify operation after downloading the hex file
        #[arg(long, default_value_t = false)]
        skip_verify: bool,
    },

    /// Rework command which can be used to setup a device
    ///
    /// This command can be used to download multiple binary files to a device
    /// - Bootloader
    /// - Application
    /// - Configuration
    ///
    /// Additionally, an optional BLE stack can be downloaded (stm32wb)
    ///
    /// Optionally, the a mass erase can be performed before downloading the files and the readout protection can be set after finishing all download operations
    Rework {
        /// A list of binary files to download. Each file is specified as `hex_address,path/to/file.bin`
        ///
        /// Example: `0x08000000,my_bin.bin`
        /// The files are downloaded in the order they are specified
        #[arg(value_parser=bin_file_info_from_cli_input, long)]
        bin_file: Vec<BinFileInfo>,

        /// Optional BLE stack binary file to download. Specified as `hex_address,path/to/file.bin`
        ///
        /// Example: `0x08000000,my_ble_stack.bin`
        /// This command will delete the existing BLE stack before downloading the new one (first install == true)
        #[arg(value_parser=bin_file_info_from_cli_input, long)]
        ble_stack: Option<BinFileInfo>,

        #[arg(value_parser=version_from_cli_input, long)]
        ble_stack_version: Option<fus::Version>,

        /// Whether to set the readout protection after downloading the files
        #[arg(long = "set-rdp", default_value_t = false)]
        set_readout_protection: bool,
    },
}

/// Sample CLI application for CubeProgrammer. Needs .env file with STM32_CUBE_PROGRAMMER_DIR set or STM32_CUBE_PROGRAMMER_DIR environment variable set
fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let logger = env_logger::Builder::new()
        .filter_level(cli.verbosity.into())
        .filter_module("stm32cubeprogrammer::api_log", log::LevelFilter::Off) // Silence the hyper crate
        .build();

    if dotenvy::dotenv().is_err() {
        log::warn!("No .env file found");
    }

    let cube_programmer_dir = env::var("STM32_CUBE_PROGRAMMER_DIR")
        .with_context(|| "STM32_CUBE_PROGRAMMER_DIR environment variable not set")?;

    let display_handler = std::sync::Arc::new(Mutex::new(DisplayHandler::new(logger)));

    let api = stm32cubeprogrammer::CubeProgrammerApi::builder()
        .cube_programmer_dir(&cube_programmer_dir)
        .display_callback(display_handler.clone())
        .build()
        .with_context(|| "Failed to create CubeProgrammer API instance")?;

    let probes = api.list_available_probes();

    if probes.is_empty() {
        return Err(anyhow::anyhow!("No ST-Link probes found"));
    }

    let selected_probe = if let Some(serial) = &cli.serial {
        probes
            .iter()
            .find(|probe| **probe.as_ref() == *serial)
            .ok_or_else(|| {
                anyhow::anyhow!("No ST-Link probe found with serial number: {}", serial)
            })?
    } else {
        log::info!("No probe serial supplied, selecting first connected probe");
        &probes[0]
    };

    let programmer = api
        .connect_to_target(
            selected_probe,
            &probe::Protocol::Swd,
            &probe::ConnectionParameters::default(),
        )
        .with_context(|| "Failed to connect to target")?;

    match &cli.command {
        Commands::Rework {
            bin_file,
            ble_stack,
            ble_stack_version,
            set_readout_protection,
        } => {
            // TODO: Read FUS information and check if read protection is enabled
            log::info!("## Rework command ##");
            log::info!("#");

            for (index, file) in bin_file.iter().enumerate() {
                log::info!("# Binary file {}: {:?}", index, file);
            }

            log::info!("#");

            if let Some(ble_stack) = ble_stack {
                log::info!("# BLE stack: {:?}", ble_stack);
            }

            log::info!("#");
            log::info!("# Set readout protection: {}", set_readout_protection);
            log::info!("#");
            log::info!("##");

            programmer
                .disable_read_out_protection()
                .with_context(|| "Failed to disable readout protection")?;

            if let Some(ble_stack) = ble_stack {
                log::info!("Start FUS and check if BLE stack needs to be flashed");
                programmer.disconnect();

                let programmer = api
                    .connect_to_target_fus(selected_probe, &probe::Protocol::Swd)
                    .with_context(|| "Cannot start FUS")?;

                let flash_ble_stack = if let Some(version) = ble_stack_version {
                    let target_version = programmer.fus_info().wireless_stack_version;
                    if target_version != *version {
                        log::info!(
                            "Version on target {} NOT EQUAL to given version {}. Flash stack",
                            target_version,
                            version
                        );

                        true
                    } else {
                        log::info!(
                            "Version on target {} EQUAL to given version {}. Skip flashing stack",
                            target_version,
                            version
                        );

                        false
                    }
                } else {
                    log::info!("No BLE stack version given. Flash stack");
                    true
                };

                if flash_ble_stack {
                    programmer
                        .update_ble_stack(&ble_stack.path, ble_stack.address.0, false, true, true)
                        .with_context(|| "Failed to update BLE stack")?;
                } else {
                    programmer
                        .start_wireless_stack()
                        .with_context(|| "Failed to start wireless stack")?;
                }

                programmer.disconnect();
            }

            let programmer = api
                .connect_to_target(
                    selected_probe,
                    &probe::Protocol::Swd,
                    &probe::ConnectionParameters::default(),
                )
                .with_context(|| "Failed to connect to target")?;

            // Download binary files
            for file in bin_file {
                log::info!("Downloading binary file: {:?}", file);
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Download binary");

                programmer
                    .download_bin_file(&file.path, file.address.0, false, true)
                    .with_context(|| "Failed to download binary file")?;
            }

            // Set readout protection
            if *set_readout_protection {
                log::info!("Setting readout protection");
                display_handler.lock().unwrap().set_message("Set RDP");

                programmer
                    .enable_read_out_protection()
                    .with_context(|| "Failed to set readout protection")?;
            }

            // Reset device
            log::info!("Resetting device");
            programmer
                .reset_target(probe::ResetMode::Hardware)
                .with_context(|| "Failed to reset device")?;

            Ok(())
        }
        Commands::DownloadHex {
            hex_file,
            skip_erase,
            skip_verify,
        } => {
            log::info!("## Download hex command ##");
            log::info!("#");
            log::info!("# Hex file: {:?}", hex_file);
            log::info!("#");
            log::info!("##");

            programmer
                .download_hex_file(hex_file, *skip_erase, *skip_verify)
                .with_context(|| "Failed to download hex file")?;

            Ok(())
        }
        Commands::DownloadBin {
            bin_file,
            skip_erase,
            skip_verify,
        } => {
            log::info!("## Download hex command ##");
            log::info!("#");
            log::info!("# Bin file info: {}", bin_file);
            log::info!("#");
            log::info!("##");

            display_handler
                .lock()
                .unwrap()
                .set_message("Download binary");

            programmer
                .download_bin_file(
                    &bin_file.path,
                    bin_file.address.0,
                    *skip_erase,
                    *skip_verify,
                )
                .with_context(|| "Failed to download hex file")?;

            Ok(())
        }
        Commands::MassErase => {
            log::info!("## Mass erase command ##");

            programmer
                .mass_erase()
                .with_context(|| "Failed to mass erase")?;

            Ok(())
        }
        Commands::Reset => {
            log::info!("## Reset command ##");

            programmer
                .reset_target(probe::ResetMode::Hardware)
                .with_context(|| "Failed to reset device")?;

            Ok(())
        }
        Commands::ReatProtection { enable } => {
            log::info!("## Read protection command ##");

            match enable {
                ReadProtection::Enable => {
                    programmer
                        .enable_read_out_protection()
                        .with_context(|| "Failed to enable read protection")?;
                }
                ReadProtection::Disable => {
                    programmer
                        .disable_read_out_protection()
                        .with_context(|| "Failed to disable read protection")?;
                }
            }

            Ok(())
        }
    }
}
