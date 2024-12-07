use derive_more::{Display, Error};
use stm32cubeprogrammer_sys::libloading;

use crate::api_types;

pub type CubeProgrammerResult<T> = std::result::Result<T, CubeProgrammerError>;

/// Add additional context why a type conversion failed
#[derive(Debug, Error, Display)]
pub enum TypeConversionError {
    Utf8Error,
    Utf16Error,
    NullError,
}

#[derive(Debug, Error, Display)]
pub enum CubeProgrammerError {
    #[display("LibLoading error: {}", _0)]
    LibLoading(libloading::Error),

    #[display("Command return code error: {}", return_code)]
    CommandReturnCode { return_code: api_types::ErrorCode },

    #[display("Command returned null error")]
    CommandReturnNull,

    #[display("Parameter error: {}", message)]
    Parameter { message: String },

    #[display("Conversion error: {}", message)]
    TypeConversion {
        message: String,

        #[error(source)]
        source: TypeConversionError,
    },

    #[display("AssertionFailed: {}", message)]
    AssertionFailed { message: String },

    #[display("Target connection lost")]
    ConnectionLost,

    #[display("File IO error: {}", _0)]
    FileIo(std::io::Error)
}
