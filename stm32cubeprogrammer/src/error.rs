use derive_more::{Display, Error};

pub type CubeProgrammerResult<T> = std::result::Result<T, CubeProgrammerError>;

/// Add additional context why a type conversion failed
#[derive(Debug, Error, Display)]
pub enum TypeConversionError {
    Utf8Error,
    Utf16Error,
    NullError,
    BytemuckError,
    VersionError,
}

#[derive(Debug, Error, Display)]
pub enum Action {
    Connect,
    ReadTargetInfo,
    ReadMemory,
    WriteMemory,
    StartFus,
    ReadFusInfo,
    Reset,
    DownloadFile,
    MassErase,
    SaveMemory,
    EnableReadOutProtection,
    DisableReadOutProtection,
    CheckConnection,
    UpgradeWirelessStack,
    DeleteWirelessStack,
    StartWirelessStack,
    ListConnectedProbes,
    WriteCoreRegister,
    ReadCoreRegister,
}

#[derive(Debug, Error, Display)]
pub enum UnexpectedOutput {
    Null,
    SliceConversion,
    SliceLength,
}

#[derive(Debug, Error, Display)]
pub enum CubeProgrammerError {
    #[display("Action {} failed with return code: {}", action, return_code)]
    ActionFailed {
        action: Action,
        return_code: crate::api_types::ErrorCode,
    },

    #[display("Action {} returns unexpected output: {}", action, unexpected_output)]
    ActionOutputUnexpected {
        action: Action,
        unexpected_output: UnexpectedOutput,
    },

    #[display("Action {} not supported: {}", action, message)]
    ActionNotSupported {
        action: Action,
        message: String,
    },

    #[display("Parameter error: {}", message)]
    Parameter {
        action: Action,
        message: String,
    },

    #[display("Conversion error: {}", message)]
    TypeConversion {
        message: String,

        #[error(source)]
        source: TypeConversionError,
    },

    FileIo(std::io::Error),

    LibLoading(stm32cubeprogrammer_sys::libloading::Error),
}
