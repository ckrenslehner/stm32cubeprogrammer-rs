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
pub enum CubeProgrammerError {
    #[display("Command return code error: {}", return_code)]
    CommandReturnCode {
        return_code: crate::api_types::ErrorCode,
    },

    #[display("Null value error: {}", message)]
    NullValue {
        message: String,
    },

    #[display("Operation not supported: {}", message)]
    NotSupported {
        message: String,
    },

    #[display("Parameter error: {}", message)]
    Parameter {
        message: String,
    },

    #[display("Conversion error: {}", message)]
    TypeConversion {
        message: String,

        #[error(source)]
        source: TypeConversionError,
    },

    #[display("Target connection lost")]
    ConnectionLost,

    #[display("File IO error: {}", _0)]
    FileIo(std::io::Error),

    LibLoading(stm32cubeprogrammer_sys::libloading::Error),
}
