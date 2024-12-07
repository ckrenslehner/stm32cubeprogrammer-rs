mod display_handler;
use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use display_handler::DisplayHandler;
use std::{env, sync::Mutex};
use stm32cubeprogrammer::CubeProgrammer;

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

impl BinFileInfo {
    fn from_cli_input(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(',').collect();

        if parts.len() != 2 {
            return Err(format!("Invalid format: {}", s));
        }

        let address = HexAddress::from_cli_input(parts[0])?;
        let path = std::path::PathBuf::from(parts[1]);

        Ok(Self { address, path })
    }
}

impl std::fmt::Display for BinFileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display the address in hex format and the path
        write!(f, "{:#x}, {:?}", self.address.0, self.path)
    }
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
        #[arg(value_parser=BinFileInfo::from_cli_input)]
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
        #[arg(value_parser=BinFileInfo::from_cli_input, long)]
        bin_file: Vec<BinFileInfo>,

        /// Optional BLE stack binary file to download. Specified as `hex_address,path/to/file.bin`
        ///
        /// Example: `0x08000000,my_ble_stack.bin`
        /// This command will delete the existing BLE stack before downloading the new one (first install == true)
        #[arg(value_parser=BinFileInfo::from_cli_input, long)]
        ble_stack: Option<BinFileInfo>,

        /// Whether to erase the device before downloading the files
        ///
        /// If this is set to false, a sector erase will be performed when downloading the files
        #[arg(long, default_value_t = false)]
        mass_erase: bool,

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

    let programmer = CubeProgrammer::builder()
        .cube_programmer_dir(cube_programmer_dir)
        .display_callback(display_handler.clone())
        .build()
        .with_context(|| "Failed to create CubeProgrammer instance")?;

    // Discover available probes
    let probes = programmer.list_connected_st_link_probes();

    if probes.is_empty() {
        return Err(anyhow::anyhow!("No ST-Link probes found"));
    }

    let selected_probe = if let Some(serial) = &cli.serial {
        probes
            .iter()
            .find(|probe| probe.serial_number() == *serial)
            .ok_or_else(|| {
                anyhow::anyhow!("No ST-Link probe found with serial number: {}", serial)
            })?
    } else {
        log::info!("No probe serial supplied, selecting first connected probe");
        &probes[0]
    };

    let selected_probe = stm32cubeprogrammer::ConnectParameters::builder()
        .base_connect_parameters(selected_probe)
        .frequency(stm32cubeprogrammer::Frequency::Highest)
        .build()
        .with_context(|| "Failed to create connect parameters")?;

    log::info!(
        "Connecting to target via selected probe: {}",
        selected_probe
    );

    let programmer = programmer
        .connect_to_target(&selected_probe)
        .with_context(|| "Failed to connect to target")?;

    match &cli.command {
        Commands::Rework {
            bin_file,
            ble_stack,
            mass_erase,
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
            log::info!("# Mass erase: {}", mass_erase);
            log::info!("# Set readout protection: {}", set_readout_protection);
            log::info!("#");
            log::info!("##");

            programmer
                .disable_read_out_protection()
                .with_context(|| "Failed to disable readout protection")?;

            // Mass erase
            if *mass_erase {
                log::info!("Mass erase");
                display_handler.lock().unwrap().set_message("Mass erase");

                programmer
                    .mass_erase()
                    .with_context(|| "Failed to mass erase")?;
            }

            // Download binary files
            for file in bin_file {
                log::info!("Downloading binary file: {:?}", file);
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Download binary");

                programmer
                    .download_bin_file(&file.path, file.address.0, *mass_erase, true)
                    .with_context(|| "Failed to download binary file")?;
            }

            // Download BLE stack
            if let Some(ble_stack) = ble_stack {
                log::info!("Downloading BLE stack: {:?}", ble_stack);
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Download BLE stack");

                programmer
                    .update_ble_stack(&ble_stack.path, ble_stack.address.0, false, true, true)
                    .with_context(|| "Failed to download BLE stack")?;
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
                .reset_target(stm32cubeprogrammer::ResetMode::HardwareReset)
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
                .reset_target(stm32cubeprogrammer::ResetMode::HardwareReset)
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
