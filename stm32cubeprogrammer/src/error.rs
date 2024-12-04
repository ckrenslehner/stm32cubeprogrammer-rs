use derive_more::{Display, Error};
use stm32cubeprogrammer_sys::libloading;

pub type CubeProgrammerApiResult<T> = std::result::Result<T, CubeProgrammerApiError>;

#[derive(Debug, Error, Display)]
pub enum CubeProgrammerApiError {
    #[display("LibLoadingError: {}", _0)]
    LibLoadingError(libloading::Error),

    #[display("CommandError: {}", return_code)]
    CommandReturnCode { return_code: i32 },

    #[display("CommandError: Command returned null")]
    CommandReturnNull,
}