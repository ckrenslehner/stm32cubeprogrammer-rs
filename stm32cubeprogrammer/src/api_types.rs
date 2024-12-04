use crate::error::{CubeProgrammerApiError, CubeProgrammerApiResult};
use derive_more::derive::{From, Into};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Return code which is mapped to an error if it is not 0
#[derive(Debug, From, Into)]
pub struct ReturnCode(pub i32);

impl ReturnCode {
    pub fn check(&self) -> CubeProgrammerApiResult<()> {
        if self.0 == 0 {
            Ok(())
        } else {
            Err(CubeProgrammerApiError::CommandReturnCode {
                return_code: self.0,
            })
        }
    }
}

#[derive(Debug, Default, Clone, Copy, IntoPrimitive, TryFromPrimitive, strum::Display)]
#[repr(i32)]
pub enum DebugPort {
    Jtag,
    Swd,

    #[default]
    Unknown = -1,
}

#[derive(Debug, Default, Clone, Copy, IntoPrimitive, TryFromPrimitive, strum::Display)]
#[repr(i32)]
pub enum ResetMode {
    SoftwareReset,
    HardwareReset,
    CoreReset,

    #[default]
    Unknown = -1,
}

#[derive(Debug, Default, Clone, Copy, IntoPrimitive, TryFromPrimitive, strum::Display)]
#[repr(i32)]
pub enum ConnectionMode {
    NormalMode,
    HotplugMode,
    UnderResetMode,
    PowerDownMode,
    HwResetPulseMode,

    #[default]
    Unknown = -1,
}

#[derive(Debug, Clone)]
pub struct StLink(pub(crate) stm32cubeprogrammer_sys::debugConnectParameters);

impl StLink {
    pub fn serial_number(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.serialNumber.as_ref())
    }

    pub fn board(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.board.as_ref())
    }

    pub fn firmware_version(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.firmwareVersion.as_ref())
    }

    pub fn debug_port(&self) -> DebugPort {
        DebugPort::try_from(self.0.dbgPort).unwrap_or(DebugPort::default())
    }

    pub fn connection_mode(&self) -> ConnectionMode {
        ConnectionMode::try_from(self.0.connectionMode).unwrap_or(ConnectionMode::default())
    }

    pub fn reset_mode(&self) -> ResetMode {
        ResetMode::try_from(self.0.resetMode).unwrap_or(ResetMode::default())
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

impl std::fmt::Display for StLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::utility::cchar_to_null_terminated_string;

        write!(
            f,
            "STLink (Serial: {}), Board: {}, Firmware version: {}, Debug port: {}, Connection mode: {}, Reset mode: {}, Shared: {}",
            cchar_to_null_terminated_string(self.0.serialNumber.as_ref()),
            cchar_to_null_terminated_string(self.0.board.as_ref()),
            cchar_to_null_terminated_string(self.0.firmwareVersion.as_ref()),
            DebugPort::try_from(self.0.dbgPort).unwrap_or(DebugPort::default()),
            ConnectionMode::try_from(self.0.connectionMode).unwrap_or(ConnectionMode::default()),
            ResetMode::try_from(self.0.resetMode).unwrap_or(ResetMode::default()),
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
        crate::utility::cchar_to_null_terminated_string(self.0.type_.as_ref())
    }

    pub fn cpu(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.cpu.as_ref())
    }

    pub fn name(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.name.as_ref())
    }

    pub fn series(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.series.as_ref())
    }

    pub fn description(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.description.as_ref())
    }

    pub fn revision_id(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.revisionId.as_ref())
    }

    pub fn board(&self) -> &str {
        crate::utility::cchar_to_null_terminated_string(self.0.board.as_ref())
    }
}

impl std::fmt::Display for TargetInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::utility::cchar_to_null_terminated_string;

        write!(
            f,
            "Target information (Device ID: {}, Flash size: {}, Bootloader version: {}, Device type: {}, CPU: {}, Name: {}, Series: {}, Description: {}, Revision ID: {}, Board: {})",
            self.0.deviceId,
            self.0.flashSize,
            self.0.bootloaderVersion,
            cchar_to_null_terminated_string(self.0.type_.as_ref()),
            cchar_to_null_terminated_string(self.0.cpu.as_ref()),
            cchar_to_null_terminated_string(self.0.name.as_ref()),
            cchar_to_null_terminated_string(self.0.series.as_ref()),
            cchar_to_null_terminated_string(self.0.description.as_ref()),
            cchar_to_null_terminated_string(self.0.revisionId.as_ref()),
            cchar_to_null_terminated_string(self.0.board.as_ref())
        )
    }
}
