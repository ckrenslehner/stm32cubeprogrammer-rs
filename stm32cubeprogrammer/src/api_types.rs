use crate::error::{CubeProgrammerError, CubeProgrammerResult};
use derive_more::derive::{AsRef, Deref, Display, From, Into};
use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Error codes returned by the CubeProgrammer API
#[derive(Debug, Copy, Clone, strum::Display, IntoPrimitive, FromPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
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

/// Return code which is mapped to an error if it is not equal to SUCCESS
/// Sometimes success is 0, sometimes it is 1
#[derive(Debug, From, Into)]
pub(crate) struct ReturnCode<const SUCCESS: i32>(pub(crate) i32);

impl<const SUCCESS: i32> ReturnCode<SUCCESS> {
    pub(crate) fn check(&self, action: crate::error::Action) -> CubeProgrammerResult<()> {
        if self.0 == SUCCESS {
            Ok(())
        } else {
            Err(CubeProgrammerError::ActionFailed {
                action,
                return_code: ErrorCode::from(self.0),
            })
        }
    }
}

pub mod probe {
    use super::*;

    #[derive(
        Debug, Default, Clone, Copy, PartialEq, IntoPrimitive, TryFromPrimitive, strum::Display,
    )]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    #[cfg_attr(windows, repr(i32))]
    #[cfg_attr(unix, repr(u32))]
    /// Debug protocol for the target connection
    pub enum Protocol {
        Jtag,
        #[default]
        Swd,
    }

    #[derive(
        Debug, Default, Clone, Copy, PartialEq, IntoPrimitive, TryFromPrimitive, strum::Display,
    )]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    #[cfg_attr(windows, repr(i32))]
    #[cfg_attr(unix, repr(u32))]
    /// Reset mode for the target connection
    pub enum ResetMode {
        Software,
        #[default]
        Hardware,
        Core,
    }

    #[derive(
        Debug, Default, Clone, Copy, PartialEq, IntoPrimitive, TryFromPrimitive, strum::Display,
    )]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    #[cfg_attr(windows, repr(i32))]
    #[cfg_attr(unix, repr(u32))]
    /// Connection mode for the target connection
    pub enum ConnectionMode {
        #[default]
        Normal,
        HotPlug,
        UnderReset,
        PowerDown,
        HardwareResetPulse,
    }

    /// Frequency for the target connection
    #[derive(Debug, Default, Clone, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    pub enum Frequency {
        Low,
        Medium,
        High,
        #[default]
        Highest,

        Custom(u32),
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    /// Connection parameters for the target connection
    pub struct ConnectionParameters {
        pub frequency: Frequency,
        pub reset_mode: ResetMode,
        pub connection_mode: ConnectionMode,
    }

    impl Default for ConnectionParameters {
        fn default() -> Self {
            Self {
                frequency: Frequency::Highest,
                reset_mode: ResetMode::Hardware,
                connection_mode: ConnectionMode::Normal,
            }
        }
    }

    #[derive(Debug, Clone, Deref, From, AsRef, Into, Hash, PartialEq, Eq, Display)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    /// The serial of a probe
    pub struct Serial(String);

    impl std::str::FromStr for Serial {
        type Err = CubeProgrammerError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.is_empty() {
                return Err(CubeProgrammerError::TypeConversion {
                    message: "Cannot convert empty string to serial".to_string(),
                    source: crate::error::TypeConversionError::NullError,
                });
            }

            Ok(Serial(s.to_string()))
        }
    }

    #[derive(Debug, Clone, Deref)]
    #[repr(transparent)]
    /// Transparent wrapper around the [`stm32cubeprogrammer_sys::debugConnectParameters`]
    pub(crate) struct Probe(pub(crate) stm32cubeprogrammer_sys::debugConnectParameters);

    impl Probe {
        /// Create a modified version of connect parameters
        pub(crate) fn new(
            probe: &Probe,
            protocol: &Protocol,
            connect_parameters: &ConnectionParameters,
        ) -> Self {
            let mut debug_probe = probe.clone();

            debug_probe.set_debug_protocol(*protocol);
            debug_probe.set_reset_mode(connect_parameters.reset_mode);
            debug_probe.set_connection_mode(connect_parameters.connection_mode);
            debug_probe.set_shared(false);

            let frequency = match (&connect_parameters.frequency, debug_probe.debug_port()) {
                (Frequency::Custom(custom_frequency), _) => Some(*custom_frequency),
                (Frequency::Low, Protocol::Jtag) => debug_probe.0.freq.jtagFreq.get(3).copied(),
                (Frequency::Low, Protocol::Swd) => debug_probe.0.freq.swdFreq.get(3).copied(),
                (Frequency::Medium, Protocol::Jtag) => debug_probe.0.freq.jtagFreq.get(2).copied(),
                (Frequency::Medium, Protocol::Swd) => debug_probe.0.freq.swdFreq.get(2).copied(),
                (Frequency::High, Protocol::Jtag) => debug_probe.0.freq.jtagFreq.get(1).copied(),
                (Frequency::High, Protocol::Swd) => debug_probe.0.freq.swdFreq.get(1).copied(),
                (Frequency::Highest, Protocol::Jtag) => {
                    debug_probe.0.freq.jtagFreq.first().copied()
                }
                (Frequency::Highest, Protocol::Swd) => debug_probe.0.freq.swdFreq.first().copied(),
            };

            debug_assert!(frequency.is_some());
            debug_probe.0.frequency = frequency.expect("Cannot get frequency") as i32;

            debug_probe
        }

        pub(crate) fn serial_number(&self) -> &str {
            crate::utility::c_char_slice_to_string(self.0.serialNumber.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
        }

        pub(crate) fn board(&self) -> &str {
            crate::utility::c_char_slice_to_string(self.0.board.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
        }

        pub(crate) fn firmware_version(&self) -> &str {
            crate::utility::c_char_slice_to_string(self.0.firmwareVersion.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
        }

        pub(crate) fn debug_port(&self) -> Protocol {
            Protocol::try_from(self.0.dbgPort).expect("Cannot convert debug port")
        }

        pub(crate) fn connection_mode(&self) -> ConnectionMode {
            ConnectionMode::try_from(self.0.connectionMode).expect("Cannot convert connection mode")
        }

        pub(crate) fn reset_mode(&self) -> ResetMode {
            ResetMode::try_from(self.0.resetMode).expect("Cannot convert reset mode")
        }

        pub(crate) fn shared(&self) -> bool {
            self.0.shared != 0
        }

        pub(crate) fn set_debug_protocol(&mut self, protocol: Protocol) {
            self.0.dbgPort = protocol.into();
        }

        pub(crate) fn set_connection_mode(&mut self, connection_mode: ConnectionMode) {
            self.0.connectionMode = connection_mode.into();
        }

        pub(crate) fn set_reset_mode(&mut self, reset_mode: ResetMode) {
            self.0.resetMode = reset_mode.into();
        }

        pub(crate) fn set_shared(&mut self, shared: bool) {
            self.0.shared = if shared { 1 } else { 0 };
        }
    }

    impl std::fmt::Display for Probe {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
            f,
            "STLink (Serial: {}), Board: {}, Firmware version: {}, Debug port: {}, Connection mode: {}, Reset mode: {}, Frequency: {} Hz, Shared: {}",
            self.serial_number(),
            self.board(),
            self.firmware_version(),
            self.debug_port(),
            self.connection_mode(),
            self.reset_mode(),
            self.0.frequency,
            self.shared()
        )
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
/// Information about the target device
pub struct GeneralInformation {
    pub device_id: u32,
    pub flash_size: u32,
    pub bootloader_version: u32,
    pub device_type: String,
    pub cpu: String,
    pub name: String,
    pub series: String,
    pub description: String,
    pub revision_id: String,
    pub probe_board: String,
    pub fus_support: bool,
}

impl From<stm32cubeprogrammer_sys::generalInf> for GeneralInformation {
    fn from(value: stm32cubeprogrammer_sys::generalInf) -> Self {
        let name = crate::utility::c_char_slice_to_string(value.name.as_ref())
            .unwrap_or("Unknown")
            .trim_matches('\0')
            .to_string();

        GeneralInformation {
            device_id: value.deviceId as u32,
            flash_size: value.flashSize as u32,
            bootloader_version: value.bootloaderVersion as u32,
            device_type: crate::utility::c_char_slice_to_string(value.type_.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
                .to_string(),
            cpu: crate::utility::c_char_slice_to_string(value.cpu.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
                .to_string(),
            name: name.clone(),
            series: crate::utility::c_char_slice_to_string(value.series.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
                .to_string(),
            description: crate::utility::c_char_slice_to_string(value.description.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
                .to_string(),
            revision_id: crate::utility::c_char_slice_to_string(value.revisionId.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
                .to_string(),
            probe_board: crate::utility::c_char_slice_to_string(value.board.as_ref())
                .unwrap_or("Unknown")
                .trim_matches('\0')
                .to_string(),
            fus_support: crate::utility::target_supports_fus(&name),
        }
    }
}

impl std::fmt::Display for GeneralInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Target information (Device ID: {}, Flash size: {}, Bootloader version: {}, Device type: {}, CPU: {}, Name: {}, Series: {}, Description: {}, Revision ID: {}, Board: {})",
            self.device_id,
            self.flash_size,
            self.bootloader_version,
            self.device_type,
            self.cpu,
            self.name,
            self.series,
            self.description,
            self.revision_id,
            self.probe_board
        )
    }
}

pub mod fus {
    use super::*;

    #[derive(Copy, Clone, Debug, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    /// Version of the FUS
    pub struct Version {
        pub major: u8,
        pub minor: u8,
        pub sub: u8,
        pub r#type: Option<u8>,
    }

    impl PartialEq for Version {
        fn eq(&self, other: &Self) -> bool {
            if let Some(r#type) = self.r#type {
                // Compare the type as well
                if let Some(other_type) = other.r#type {
                    self.major == other.major
                        && self.minor == other.minor
                        && self.sub == other.sub
                        && r#type == other_type
                } else {
                    false
                }
            } else {
                // Do not compare the type
                self.major == other.major && self.minor == other.minor && self.sub == other.sub
            }
        }
    }

    impl std::fmt::Display for Version {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(r#type) = self.r#type {
                write!(f, "{}.{}.{}.{}", self.major, self.minor, self.sub, r#type)
            } else {
                write!(f, "{}.{}.{}", self.major, self.minor, self.sub)
            }
        }
    }

    impl std::str::FromStr for Version {
        type Err = CubeProgrammerError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let parts = s.split('.');

            if parts.clone().count() == 3 {
                if let Ok(converted) = parts
                    .map(|x| x.parse::<u8>())
                    .collect::<Result<Vec<u8>, _>>()
                {
                    return Ok(Version {
                        major: converted[0],
                        minor: converted[1],
                        sub: converted[2],
                        r#type: None,
                    });
                }
            } else if parts.clone().count() == 4 {
                if let Ok(converted) = parts
                    .map(|x| x.parse::<u8>())
                    .collect::<Result<Vec<u8>, _>>()
                {
                    return Ok(Version {
                        major: converted[0],
                        minor: converted[1],
                        sub: converted[2],
                        r#type: Some(converted[3]),
                    });
                }
            }

            Err(CubeProgrammerError::TypeConversion {
                message: format!("Cannot convert \"{}\" to a version. Expecting the following format \"u8.u8.u8\" e.g. \"1.2.3\"", s),
                source:  crate::error::TypeConversionError::VersionError
            })
        }
    }

    #[derive(Copy, Clone, Debug, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    /// Information about the FUS. This is read from the target after a successful connection to the FUS
    pub struct Information {
        pub wireless_stack_version: Version,
        pub fus_version: Version,
        pub uid64: u64,
        pub device_id: u16,
    }

    impl std::fmt::Display for Information {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Wireless stack version: {}, FUS version: {}, UUID64: {:X}, Device ID: {:X}",
                self.wireless_stack_version, self.fus_version, self.uid64, self.device_id
            )
        }
    }
}
