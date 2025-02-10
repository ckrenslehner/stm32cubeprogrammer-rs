use serde::Serialize;
use std::env::ArgsOs;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub schema_version: String,
    pub args: String,
    pub cube_programmer_dir: std::path::PathBuf,
    pub connected_probes: Option<Vec<stm32cubeprogrammer::probe::Serial>>,
    pub selected_probe: Option<stm32cubeprogrammer::probe::Serial>,
    pub general_information: Option<stm32cubeprogrammer::api_types::GeneralInformation>,
    pub command_output: Option<Vec<CommandOutput>>,
}

impl Output {
    const SCHEMA_VERSION: &str = "1";

    pub fn new(args: ArgsOs, cube_programmer_dir: &std::path::Path) -> Self {
        let schema_version = Self::SCHEMA_VERSION.to_string();
        let args = args
            .skip(1)
            .map(|x| x.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ");

        Self {
            schema_version,
            args,
            cube_programmer_dir: cube_programmer_dir.to_path_buf(),
            connected_probes: None,
            selected_probe: None,
            general_information: None,
            command_output: None,
        }
    }

    /// Add which probe is selected
    pub fn add_selected_probe(&mut self, probe: &stm32cubeprogrammer::probe::Serial) {
        self.selected_probe = Some(probe.clone());
    }

    /// Add information about the connected target
    pub fn add_general_information(
        &mut self,
        general_information: &stm32cubeprogrammer::api_types::GeneralInformation,
    ) {
        self.general_information = Some(general_information.clone());
    }

    /// Add list of connected probes
    pub fn add_probe_list(&mut self, list: &[stm32cubeprogrammer::probe::Serial]) {
        self.connected_probes = Some(list.to_vec());
    }

    /// Add output of a command
    pub fn add_command_output(&mut self, command: CommandOutput) {
        if let Some(ref mut command_output) = self.command_output {
            command_output.push(command);
        } else {
            self.command_output = Some(vec![command]);
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "command")]
#[serde(rename_all = "camelCase")]
pub enum CommandOutput {
    #[serde(rename_all = "camelCase")]
    FlashBin {
        file: std::path::PathBuf,
        address: u32,
    },
    #[serde(rename_all = "camelCase")]
    FlashHex {
        file: std::path::PathBuf,
    },
    #[serde(rename_all = "camelCase")]
    UpdateBleStack {
        file: std::path::PathBuf,
        address: u32,
        ble_stack_updated: BleStackUpdated,
    },
    #[serde(rename_all = "camelCase")]
    BleStackInfo(stm32cubeprogrammer::fus::Information),
    #[serde(rename_all = "camelCase")]
    Reset {
        reset_mode: stm32cubeprogrammer::api_types::probe::ResetMode,
    },
    MassErase,
    Protect,
    Unprotect,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BleStackUpdated {
    #[serde(rename_all = "camelCase")]
    NotUpdated,
    #[serde(rename_all = "camelCase")]
    Updated(BleStackUpdateReason),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BleStackUpdateReason {
    #[serde(rename_all = "camelCase")]
    Force,
    #[serde(rename_all = "camelCase")]
    VersionNotEqual {
        expected: stm32cubeprogrammer::fus::Version,
        on_target: stm32cubeprogrammer::fus::Version,
    },
    #[serde(rename_all = "camelCase")]
    NoVersionProvided,
}
