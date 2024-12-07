use crate::error::{CubeProgrammerError, CubeProgrammerResult};
use bon::bon;
use derive_more::derive::{From, Into};
use num_enum::{FromPrimitive, IntoPrimitive};

/// Negative error codes returned by the CubeProgrammer API
#[derive(Debug, Copy, Clone, strum::Display, IntoPrimitive, FromPrimitive)]
#[repr(i32)]
pub enum ErrorCode {
    #[num_enum(catch_all)]
    Unknown(i32),

    DeviceNotConnected = -1,
    NoDeviceFound = -2,
    ConnectionError = -3,
    FileNotFound = -4,
    UnsupportedOperation = -5,
    UnsupportedInterface = -6,
    InsufficientMemory = -7,
    UnknownParameters = -8,
    MemoryReadError = -9,
    MemoryWriteError = -10,
    MemoryEraseError = -11,
    UnsupportedFileFormat = -12,
    RefreshRequired = -13,
    SecurityError = -14,
    FrequencyError = -15,
    RdpEnabledError = -16,
    UnknownError = -17,
}

#[test]
fn test() {
    let error_code = ErrorCode::from(10i32);
    dbg!(error_code);

    let error_code = ErrorCode::from(10i32);
    dbg!(error_code);
}

/// Return code which is mapped to an error if it is not equal to SUCCESS
/// Sometimes success is 0, sometimes it is 1
#[derive(Debug, From, Into)]
pub(crate) struct ReturnCode<const SUCCESS: i32>(pub i32);

impl<const SUCCESS: i32> ReturnCode<SUCCESS> {
    pub fn check(&self) -> CubeProgrammerResult<()> {
        if self.0 == SUCCESS {
            Ok(())
        } else {
            Err(CubeProgrammerError::CommandReturnCode {
                return_code: ErrorCode::from(self.0),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive, FromPrimitive, strum::Display)]
#[cfg_attr(windows, repr(i32))]
#[cfg_attr(unix, repr(u32))]
pub enum DebugPort {
    Jtag,
    Swd,

    #[num_enum(catch_all)]
    #[cfg(windows)]
    Unknown(i32),

    #[num_enum(catch_all)]
    #[cfg(unix)]
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive, FromPrimitive, strum::Display)]
#[cfg_attr(windows, repr(i32))]
#[cfg_attr(unix, repr(u32))]
pub enum ResetMode {
    SoftwareReset,
    HardwareReset,
    CoreReset,

    #[num_enum(catch_all)]
    #[cfg(windows)]
    Unknown(i32),

    #[num_enum(catch_all)]
    #[cfg(unix)]
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive, FromPrimitive, strum::Display)]
#[cfg_attr(windows, repr(i32))]
#[cfg_attr(unix, repr(u32))]
pub enum ConnectionMode {
    NormalMode,
    HotplugMode,
    UnderResetMode,
    PowerDownMode,
    HwResetPulseMode,

    #[num_enum(catch_all)]
    #[cfg(windows)]
    Unknown(i32),

    #[num_enum(catch_all)]
    #[cfg(unix)]
    Unknown(u32),
}

/// Frequency of the programmer (Low, Medium, High or Custom) depending on the chosen DebugPort
/// - Low: Lowest available frequency
/// - Medium: Medium frequency
/// - High: Highest available frequency
/// Custom frequency is in Hz
pub enum Frequency {
    Low,
    Medium,
    High,
    Highest,

    Custom(u32),
}

#[derive(Debug, Clone)]
pub struct ConnectParameters(pub(crate) stm32cubeprogrammer_sys::debugConnectParameters);

#[bon]
impl ConnectParameters {
    /// Create a modified version of connect parameters
    /// Needs to called with base parameters
    #[builder]
    pub fn new(
        base_connect_parameters: &ConnectParameters,
        debug_port: Option<DebugPort>,
        frequency: Option<Frequency>,
        reset_mode: Option<ResetMode>,
        connection_mode: Option<ConnectionMode>,
        shared: Option<bool>,
    ) -> Result<Self, CubeProgrammerError> {
        let mut stlink = base_connect_parameters.clone();

        if let Some(debug_port) = debug_port {
            if let DebugPort::Unknown(_) = debug_port {
                return Err(CubeProgrammerError::Parameter {
                    message: "Debug port cannot be unknown".to_owned(),
                });
            }

            stlink.set_debug_port(debug_port);
        }

        if let Some(reset_mode) = reset_mode {
            if let ResetMode::Unknown(_) = reset_mode {
                return Err(CubeProgrammerError::Parameter {
                    message: "Reset mode cannot be unknown".to_owned(),
                });
            }

            stlink.set_reset_mode(reset_mode);
        }

        if let Some(connection_mode) = connection_mode {
            if let ConnectionMode::Unknown(_) = connection_mode {
                return Err(CubeProgrammerError::Parameter {
                    message: "Connection mode cannot be unknown".to_owned(),
                });
            }

            stlink.set_connection_mode(connection_mode);
        }

        if let Some(shared) = shared {
            stlink.set_shared(shared);
        }

        if let Some(frequency) = frequency {
            let frequency = match (frequency, stlink.debug_port()) {
                (Frequency::Custom(custom_frequency), _) => Some(custom_frequency),
                (Frequency::Low, DebugPort::Jtag) => {
                    base_connect_parameters.0.freq.jtagFreq.get(3).copied()
                }
                (Frequency::Low, DebugPort::Swd) => {
                    base_connect_parameters.0.freq.swdFreq.get(3).copied()
                }
                (Frequency::Medium, DebugPort::Jtag) => {
                    base_connect_parameters.0.freq.jtagFreq.get(2).copied()
                }
                (Frequency::Medium, DebugPort::Swd) => {
                    base_connect_parameters.0.freq.swdFreq.get(2).copied()
                }
                (Frequency::High, DebugPort::Jtag) => {
                    base_connect_parameters.0.freq.jtagFreq.get(1).copied()
                }
                (Frequency::High, DebugPort::Swd) => {
                    base_connect_parameters.0.freq.swdFreq.get(1).copied()
                }
                (Frequency::Highest, DebugPort::Jtag) => {
                    base_connect_parameters.0.freq.jtagFreq.first().copied()
                }
                (Frequency::Highest, DebugPort::Swd) => {
                    base_connect_parameters.0.freq.swdFreq.first().copied()
                }
                _ => unreachable!(),
            };

            let Some(frequency) = frequency else {
                return Err(CubeProgrammerError::AssertionFailed {
                    message: "Unexpected frequency error".to_owned(),
                });
            };

            stlink.0.frequency = frequency as i32;
        }

        Ok(stlink)
    }

    pub fn serial_number(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.serialNumber.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn board(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.board.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn firmware_version(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.firmwareVersion.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn debug_port(&self) -> DebugPort {
        DebugPort::from(self.0.dbgPort)
    }

    pub fn connection_mode(&self) -> ConnectionMode {
        ConnectionMode::from(self.0.connectionMode)
    }

    pub fn reset_mode(&self) -> ResetMode {
        ResetMode::from(self.0.resetMode)
    }

    pub fn shared(&self) -> bool {
        self.0.shared != 0
    }

    pub fn set_debug_port(&mut self, debug_port: DebugPort) {
        self.0.dbgPort = debug_port.into();
    }

    pub fn set_connection_mode(&mut self, connection_mode: ConnectionMode) {
        self.0.connectionMode = connection_mode.into();
    }

    pub fn set_reset_mode(&mut self, reset_mode: ResetMode) {
        self.0.resetMode = reset_mode.into();
    }

    pub fn set_shared(&mut self, shared: bool) {
        self.0.shared = if shared { 1 } else { 0 };
    }
}

impl std::fmt::Display for ConnectParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "STLink (Serial: {}), Board: {}, Firmware version: {}, Debug port: {}, Connection mode: {}, Reset mode: {}, Frequency: {} Hz, Shared: {}",
            self.serial_number(),
            self.board(),
            self.firmware_version(),
            DebugPort::from(self.0.dbgPort),
            ConnectionMode::from(self.0.connectionMode),
            ResetMode::from(self.0.resetMode),
            self.0.frequency,
            self.0.shared
        )
    }
}

#[derive(Debug, Clone)]
pub struct TargetInformation(pub(crate) stm32cubeprogrammer_sys::generalInf);

impl TargetInformation {
    pub fn device_id(&self) -> u32 {
        self.0.deviceId as u32
    }

    pub fn flash_size(&self) -> u32 {
        self.0.flashSize as u32
    }

    pub fn bootloader_version(&self) -> u32 {
        self.0.bootloaderVersion as u32
    }

    pub fn device_type(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.type_.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn cpu(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.cpu.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn name(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.name.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn series(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.series.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn description(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.description.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn revision_id(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.revisionId.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }

    pub fn board(&self) -> &str {
        crate::utility::c_char_slice_to_string(self.0.board.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
    }
}

impl std::fmt::Display for TargetInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Target information (Device ID: {}, Flash size: {}, Bootloader version: {}, Device type: {}, CPU: {}, Name: {}, Series: {}, Description: {}, Revision ID: {}, Board: {})",
            self.device_id(),
            self.flash_size(),
            self.bootloader_version(),
            self.device_type(),
            self.cpu(),
            self.name(),
            self.series(),
            self.description(),
            self.revision_id(),
            self.board()
        )
    }
}
