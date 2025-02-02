use bpaf::*;
use stm32cubeprogrammer::utility::HexAddress;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
/// Overall arguments for CLI
/// Note: Be careful with duplicate short or long flag names in the subcommands and the parent struct
pub struct Options {
    /// Increase verbosity. Standard verbosity is info, -v is debug, -vv is trace.
    /// To disable all output except of errors, use the --quiet flag
    #[bpaf(short('v'), long("verbose"), req_flag(()), count, map(|l| {
        match l {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    }))]
    pub verbose: log::LevelFilter,

    #[bpaf(long, fallback(false))]
    pub quiet: bool,

    /// Path to the STM32CubeProgrammer root directory (e.g. `C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer`)
    #[bpaf(long, env("STM32_CUBE_PROGRAMMER_DIR"))]
    pub stm32_cube_programmer_dir: std::path::PathBuf,

    #[bpaf(long("serial"))]
    /// The serial number of the probe to use. If no serial is provided, the first connected probe will be used
    pub probe_serial: Option<stm32cubeprogrammer::probe::Serial>,

    #[bpaf(long, fallback(Protocol::Swd))]
    /// The protocol to use for communication with the target
    pub protocol: Protocol,

    #[bpaf(long("list"))]
    /// List available probes. If this flag is set, no other commands will be executed
    pub list_probes: bool,

    #[bpaf(external(target_command), many)]
    /// Commands to run
    pub target_commands: Vec<TargetCommand>,
}

#[derive(Debug, Clone, Bpaf, PartialEq)]
/// Commands
pub enum TargetCommand {
    #[bpaf(command, adjacent)]
    /// Flash binary file to target
    FlashBin(#[bpaf(external(bin_file_info))] BinFileInfo),

    #[bpaf(command, adjacent)]
    /// Flash hex file to target
    FlashHex {
        #[bpaf(long)]
        /// The path to the intel hex file
        file: std::path::PathBuf,
    },

    #[bpaf(command, adjacent)]
    /// Update the BLE stack on the target
    UpdateBleStack(#[bpaf(external(ble_stack_info))] BleStackInfo),

    /// Read BLE stack information
    #[bpaf(command, adjacent)]
    BleStackInfo {
        #[bpaf(long, fallback(None))]
        /// The flash address in format 0x123 or 0X123 where file should be written
        compare: Option<stm32cubeprogrammer::fus::Version>,
    },

    #[bpaf(command, adjacent)]
    /// Reset the target
    Reset(#[bpaf(fallback(ResetMode::Hardware), external(reset_mode))] ResetMode),

    #[bpaf(command, adjacent)]
    /// Perform a mass erase on the target flash memory
    MassErase,

    #[bpaf(command, adjacent)]
    /// Enable read protection (lvl 1) on the target
    Protect,

    #[bpaf(command, adjacent)]
    /// Disable read protection on the target
    Unprotect,
}

#[derive(Debug, Clone, Bpaf, PartialEq)]
/// Binary file info
pub struct BinFileInfo {
    #[bpaf(long)]
    /// The path to the bin file
    pub file: std::path::PathBuf,

    #[bpaf(long)]
    /// The flash address in format 0x123 or 0X123 where file should be written
    pub address: HexAddress,
}

impl std::fmt::Display for BinFileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Binary file: `{}` ; Address: 0x{:x}",
            self.file.display(),
            self.address.0
        )
    }
}

#[derive(Debug, Clone, Bpaf, PartialEq)]
/// Ble stack update info
pub struct BleStackInfo {
    #[bpaf(long)]
    /// The path to the BLE stack binary file
    pub file: std::path::PathBuf,

    #[bpaf(long)]
    /// The flash address in format 0x123 or 0X123 where file should be written
    pub address: HexAddress,

    #[bpaf(long)]
    /// Optional version of the given BLE stack in format "Major.Minor.Sub" (e.g. "1.17.0").
    /// If you also want to compare the type, you can use "Major.Minor.Sub.Type" (e.g. "1.17.0.2").
    /// If the version on the target matches this version, the BLE stack will not be updated.
    /// If no version is provided, or the version is different, the BLE stack will be updated
    pub version: Option<stm32cubeprogrammer::fus::Version>,

    #[bpaf(long)]
    /// Force the update of the BLE stack even if the version matches
    pub force: bool,
}

impl std::fmt::Display for BleStackInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BLE stack file: `{}` ; Address: 0x{:x} ; Stack version: {}, Force update: {}",
            self.file.display(),
            self.address.0,
            self.version
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "Not given".to_string()),
            self.force
        )
    }
}

#[derive(Debug, Clone, Bpaf, PartialEq)]
pub enum ResetMode {
    /// Hardware reset
    Hardware,
    /// Software reset
    Software,
    /// Core reset
    Core,
}

impl From<ResetMode> for stm32cubeprogrammer::probe::ResetMode {
    fn from(value: ResetMode) -> Self {
        match value {
            ResetMode::Hardware => stm32cubeprogrammer::probe::ResetMode::Hardware,
            ResetMode::Software => stm32cubeprogrammer::probe::ResetMode::Software,
            ResetMode::Core => stm32cubeprogrammer::probe::ResetMode::Core,
        }
    }
}

#[derive(Debug, Clone, Bpaf, PartialEq)]
pub enum Protocol {
    /// Swd
    Swd,
    /// Jtag
    Jtag,
}

impl std::str::FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "swd" => Ok(Protocol::Swd),
            "jtag" => Ok(Protocol::Jtag),
            _ => Err("Invalid protocol".to_string()),
        }
    }
}

impl From<Protocol> for stm32cubeprogrammer::probe::Protocol {
    fn from(value: Protocol) -> Self {
        match value {
            Protocol::Swd => stm32cubeprogrammer::probe::Protocol::Swd,
            Protocol::Jtag => stm32cubeprogrammer::probe::Protocol::Jtag,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_reset() {
        dotenvy::dotenv().expect("Failed to load .env file");

        let value = options().run_inner(&["reset"]).unwrap();
        println!("{:?}", value);
        assert_eq!(
            value.target_commands,
            vec![TargetCommand::Reset(ResetMode::Hardware)]
        );

        let value = options().run_inner(&["reset", "--hardware"]).unwrap();
        println!("{:?}", value);
        assert_eq!(
            value.target_commands,
            vec![TargetCommand::Reset(ResetMode::Hardware)]
        );

        let value = options().run_inner(&["reset", "--software"]).unwrap();
        println!("{:?}", value);
        assert_eq!(
            value.target_commands,
            vec![TargetCommand::Reset(ResetMode::Software)]
        );

        let value = options().run_inner(&["reset", "--core"]).unwrap();
        println!("{:?}", value);
        assert_eq!(
            value.target_commands,
            vec![TargetCommand::Reset(ResetMode::Core)]
        );
    }

    #[test]
    fn parse_multi() {
        dotenvy::dotenv().expect("Failed to load .env file");

        let value = options()
            .run_inner(&[
                "--stm32-cube-programmer-dir",
                "some/dir",
                "unprotect",
                "update-ble-stack",
                "--file",
                "stack.bin",
                "--address",
                "0x123",
                "flash-bin",
                "--file",
                "app.bin",
                "--address",
                "0x456",
                "protect",
                "reset",
            ])
            .unwrap();

        println!("{:?}", value);

        assert_eq!(
            value.target_commands,
            vec![
                TargetCommand::Unprotect,
                TargetCommand::UpdateBleStack(BleStackInfo {
                    file: std::path::PathBuf::from("stack.bin"),
                    address: HexAddress(0x123),
                    version: None,
                    force: false,
                }),
                TargetCommand::FlashBin(BinFileInfo {
                    file: std::path::PathBuf::from("app.bin"),
                    address: HexAddress(0x456),
                }),
                TargetCommand::Protect,
                TargetCommand::Reset(ResetMode::Hardware),
            ]
        )
    }
}
