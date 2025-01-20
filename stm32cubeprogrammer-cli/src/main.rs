//! This CLI aims to provide a simple interface for setting up stm32 targets.
//! # Supported commands
//! - Flashing bin and hex files
//! - Updating BLE stack
//! - Resetting target
//! - Mass erasing target
//! - Enabling read protection
//! - Disabling read protection
//! - Resetting target
//! 
//! All commands above can be combined in a single command line invocation by chaining them.
//! 
//! If you need other commands, feel free to open an issue or a pull request. :smile:
//!
//! # Example usage:
//! Where `installation_dir` is the path to the root directory of the STM32CubeProgrammer installation.
//! E.g. `C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer`
//!
//! ```sh
//! stm32cubeprogrammer-cli --stm32-cube-programmer-dir `installation_dir` reset
//! ```
//!
//! You can also pass the directory to the STM32CubeProgrammer installation using the `STM32_CUBE_PROGRAMMER_DIR` environment variable.
//! ```sh
//! STM32_CUBE_PROGRAMMER_DIR=`installation_dir` stm32cubeprogrammer-cli reset
//! ```
//! You can chain the different supported commands.
//! ```sh
//! STM32_CUBE_PROGRAMMER_DIR=`installation_dir` stm32cubeprogrammer-cli unprotect reset flash-hex `path_to_hex_file` protect
//! ```
//! 
//! Use the `--list` flag to list available probes.
//! ```sh
//! stm32cubeprogrammer-cli --stm32-cube-programmer-dir `installation_dir` --list
//! ```
//!
//! Use `--help` to see all available options.
//! ```sh
//! stm32cubeprogrammer-cli --help
//! ```
//! # Requirements
//! There needs to be a Stm32CubeProgrammer installation on your system. The crates are tested using Stm32CubeProgrammer version 2.18.0.
//!
//! # Platform support
//! Windows and Linux are supported and tested.
//!
//! # Warranty
//! This crate is supplied as is without any warranty. Use at your own risk.

mod display_handler;
mod parse;

use anyhow::Context;
use display_handler::DisplayHandler;
use log::{error, info};
use std::sync::Mutex;
use stm32cubeprogrammer::probe;

#[derive(Default)]
/// Track the connection state of the programmer
enum ConnectionState<'a> {
    #[default]
    Disconnected,
    Connected(Option<stm32cubeprogrammer::ConnectedProgrammer<'a>>),
    ConnectedFus(Option<stm32cubeprogrammer::ConnectedFusProgrammer<'a>>),
}

/// Helper struct to manage the programmer connection
struct ProgrammerConnection<'a> {
    api: &'a stm32cubeprogrammer::CubeProgrammer,
    connection_state: ConnectionState<'a>,
    probe_serial: &'a stm32cubeprogrammer::probe::Serial,
    probe_connection_parameters: stm32cubeprogrammer::probe::ConnectionParameters,
    protocol: stm32cubeprogrammer::probe::Protocol,
}

impl<'a> ProgrammerConnection<'a> {
    fn new(
        api: &'a stm32cubeprogrammer::CubeProgrammer,
        probe_serial: &'a stm32cubeprogrammer::probe::Serial,
        protocol: stm32cubeprogrammer::probe::Protocol,
    ) -> Self {
        Self {
            api,
            connection_state: ConnectionState::Disconnected,
            probe_serial,
            probe_connection_parameters: stm32cubeprogrammer::probe::ConnectionParameters::default(
            ),
            protocol,
        }
    }

    /// Try to get a reference to a connected programmer no matter the current connection state
    fn connection(&mut self) -> Result<&stm32cubeprogrammer::ConnectedProgrammer, anyhow::Error> {
        match &mut self.connection_state {
            ConnectionState::Disconnected => {
                // Connect to the target directly
                let programmer = self
                    .api
                    .connect_to_target(
                        self.probe_serial,
                        &self.protocol,
                        &self.probe_connection_parameters,
                    )
                    .with_context(|| "Failed to connect to target")?;

                self.connection_state = ConnectionState::Connected(Some(programmer));
            }
            ConnectionState::ConnectedFus(connected_fus_cube_programmer) => {
                // Disconnect from FUS and reconnect to the target
                let inner = connected_fus_cube_programmer.take().unwrap();
                inner.disconnect();

                let programmer = self
                    .api
                    .connect_to_target(
                        self.probe_serial,
                        &self.protocol,
                        &self.probe_connection_parameters,
                    )
                    .with_context(|| "Failed to connect to target")?;

                self.connection_state = ConnectionState::Connected(Some(programmer));
            }
            _ => {}
        }

        match &self.connection_state {
            ConnectionState::Connected(programmer) => Ok(programmer.as_ref().unwrap()),
            _ => unreachable!(),
        }
    }

    /// Try to get a reference to a connected FUS programmer no matter the current connection state
    fn fus_connection(
        &mut self,
    ) -> Result<&stm32cubeprogrammer::ConnectedFusProgrammer, anyhow::Error> {
        match &mut self.connection_state {
            ConnectionState::Disconnected => {
                // Connect to FUS directly
                let programmer = self
                    .api
                    .connect_to_target_fus(self.probe_serial, &probe::Protocol::Swd)
                    .with_context(|| "Failed to connect to fus target")?;

                self.connection_state = ConnectionState::ConnectedFus(Some(programmer));
            }
            ConnectionState::Connected(connected_programmer) => {
                // Disconnect and reconnect to FUS
                let inner = connected_programmer.take().unwrap();
                inner.disconnect();

                let programmer = self
                    .api
                    .connect_to_target_fus(self.probe_serial, &probe::Protocol::Swd)
                    .with_context(|| "Failed to connect to target")?;

                self.connection_state = ConnectionState::ConnectedFus(Some(programmer));
            }
            _ => {}
        }

        match &self.connection_state {
            ConnectionState::ConnectedFus(programmer) => Ok(programmer.as_ref().unwrap()),
            _ => unreachable!(),
        }
    }
}

/// Select a probe based on the provided serial number or use the first connected probe
fn select_probe<'a>(
    probes: &'a [stm32cubeprogrammer::probe::Serial],
    probe_serial: &'a Option<stm32cubeprogrammer::probe::Serial>,
) -> Result<&'a stm32cubeprogrammer::probe::Serial, anyhow::Error> {
    if let Some(serial) = probe_serial {
        probes.iter().find(|probe| *probe == serial).ok_or_else(|| {
            error!("No ST-Link probe found with serial number: {}", serial);
            anyhow::anyhow!("No ST-Link probe found with serial number: {}", serial)
        })
    } else {
        log::info!("No probe serial provided. Using the first connected probe");
        Ok(&probes[0])
    }
}

/// Initialize the display handler
fn init_display_handler(verbosity: log::LevelFilter) -> std::sync::Arc<Mutex<DisplayHandler>> {
    let logger = env_logger::Builder::new()
        .filter_level(verbosity)
        .filter_module("stm32cubeprogrammer::api_log", log::LevelFilter::Off) // Silence the hyper crate
        .build();

    std::sync::Arc::new(Mutex::new(DisplayHandler::new(logger)))
}

/// Main entry point of the CLI
fn main() -> Result<(), anyhow::Error> {
    // Parse command line arguments
    let options = parse::options().run();

    // Init api
    let display_handler = init_display_handler(options.verbosity);
    let api = stm32cubeprogrammer::CubeProgrammer::builder()
        .cube_programmer_dir(&options.stm32_cube_programmer_dir)
        .display_callback(display_handler.clone())
        .build()
        .with_context(|| "Failed to create CubeProgrammer API instance")?;

    // Scan for probes
    let probes = api
        .list_available_probes()
        .with_context(|| "Failed to list available probes")?;

    // Early return if the list_probes flag is set
    if options.list_probes {
        if probes.is_empty() {
            info!("No ST-Link probes found");
        } else {
            for probe in &probes {
                info!("{}", probe);
            }
        }

        return Ok(());
    }

    // Select a probe
    if probes.is_empty() {
        error!("No ST-Link probes found");
        return Err(anyhow::anyhow!("No ST-Link probes found"));
    }

    let selected_probe = select_probe(&probes, &options.probe_serial)?;

    // Create a managed connection
    let mut programmer_connection =
        ProgrammerConnection::new(&api, selected_probe, options.protocol.into());

    programmer_connection.connection().map_err(|x| {
        error!("Failed to connect to target: {:?}", x);
        x
    })?;

    // Check if the command list includes a fus command and if so, check if the current target even supports FUS
    // Early return if the target does not support FUS
    if options
        .target_commands
        .iter()
        .any(|command| matches!(command, parse::TargetCommand::UpdateBleStack(_)))
        && !programmer_connection.connection()?.fus_support()
    {
        error!("The target does not support FUS commands");
        return Err(anyhow::anyhow!("The target does not support FUS commands"));
    }

    // Handle commands
    for command in options.target_commands {
        match command {
            parse::TargetCommand::FlashBin(bin_file_info) => {
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Flashing binary file");

                programmer_connection
                    .connection()?
                    .download_bin_file(bin_file_info.file, bin_file_info.address.0, false, true)
                    .with_context(|| "Failed to flash binary file")?;
            }
            parse::TargetCommand::FlashHex { file } => {
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Flashing hex file");

                programmer_connection
                    .connection()?
                    .download_hex_file(file, false, true)
                    .with_context(|| "Failed to flash hex file")?;
            }
            parse::TargetCommand::UpdateBleStack(ble_stack_info) => {
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Updating BLE stack");

                let fus_programmer = programmer_connection.fus_connection()?;

                let flash = if let Some(stack_version) = ble_stack_info.version {
                    if fus_programmer.fus_info().wireless_stack_version == stack_version {
                        log::info!("BLE stack is already up to date");
                        false
                    } else {
                        log::info!(
                            "Versions not equal. Current BLE stack version: {}",
                            fus_programmer.fus_info().wireless_stack_version
                        );
                        log::info!("Updating BLE stack to version {}", stack_version);
                        true
                    }
                } else {
                    true
                };

                if flash {
                    fus_programmer
                        .upgrade_wireless_stack(
                            ble_stack_info.file,
                            ble_stack_info.address.0,
                            false,
                            true,
                            true,
                        )
                        .with_context(|| "Failed to update BLE stack")?;
                } else {
                    fus_programmer
                        .start_wireless_stack()
                        .with_context(|| "Failed to start BLE stack")?;
                }
            }
            parse::TargetCommand::Reset(reset_mode) => {
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Resetting target");

                programmer_connection
                    .connection()?
                    .reset_target(reset_mode.into())
                    .with_context(|| "Failed to reset target")?;
            }
            parse::TargetCommand::MassErase => {
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Mass erasing target");

                programmer_connection
                    .connection()?
                    .mass_erase()
                    .with_context(|| "Failed to mass erase target")?;
            }
            parse::TargetCommand::Protect => {
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Enabling read protection");

                programmer_connection
                    .connection()?
                    .enable_read_out_protection()
                    .with_context(|| "Failed to enable read protection")?;
            }
            parse::TargetCommand::Unprotect => {
                display_handler
                    .lock()
                    .unwrap()
                    .set_message("Disabling read protection");

                programmer_connection
                    .connection()?
                    .disable_read_out_protection()
                    .with_context(|| "Failed to disable read protection")?;
            }
        }

        display_handler.lock().unwrap().set_finish();
    }

    Ok(())
}
