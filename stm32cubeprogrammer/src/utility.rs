use core::str;
use std::ffi::{c_char, CString};
use std::path::Path;

use crate::error::{CubeProgrammerError, CubeProgrammerResult, TypeConversionError};

#[derive(Debug, Clone, PartialEq)]
/// Wrapper type for parsing a hex address from a string
pub struct HexAddress(pub u32);

impl std::str::FromStr for HexAddress
where
    HexAddress: Sized,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = if s.starts_with("0x") || s.starts_with("0X") {
            u32::from_str_radix(&s[2..], 16).map_err(|_| "Failed to parse hex address")?
        } else {
            u32::from_str_radix(s, 16).map_err(|_| "Failed to parse hex address")?
        };
        Ok(HexAddress(address))
    }
}

/// Remove the extended path prefix from a path
fn remove_extended_path_prefix(path: &str) -> &str {
    #[cfg(windows)]
    const EXTENDED_PATH_PREFIX: &str = "\\\\?\\";

    #[cfg(windows)]
    if path.starts_with(EXTENDED_PATH_PREFIX) {
        path.strip_prefix(EXTENDED_PATH_PREFIX).unwrap()
    } else {
        path
    }

    #[cfg(not(windows))]
    path
}

/// Convert a path to cstring
/// If the path is a extended length path, the prefix will be removed
pub(crate) fn path_to_cstring(path: impl AsRef<Path>) -> CubeProgrammerResult<CString> {
    let path = path
        .as_ref()
        .to_str()
        .ok_or(CubeProgrammerError::TypeConversion {
            message: format!("Cannot convert path `{:?}` to string", path.as_ref()),
            source: TypeConversionError::Utf8Error,
        })?;

    string_to_cstring(remove_extended_path_prefix(path))
}

/// Convert a path to a wide string
/// If the path is a extended length path, the prefix will be removed
pub(crate) fn path_to_widestring(
    path: impl AsRef<Path>,
) -> CubeProgrammerResult<widestring::WideCString> {
    let path = path
        .as_ref()
        .to_str()
        .ok_or(CubeProgrammerError::TypeConversion {
            message: format!("Cannot convert path `{:?}` to string", path.as_ref()),
            source: TypeConversionError::Utf8Error,
        })?;

    string_to_widestring(remove_extended_path_prefix(path))
}

/// Convert a string to a wide string
pub(crate) fn string_to_widestring(s: &str) -> CubeProgrammerResult<widestring::WideCString> {
    widestring::WideCString::from_str(s).map_err(|x| CubeProgrammerError::TypeConversion {
        message: format!("Cannot convert string to widestring: {:?}", x),
        source: TypeConversionError::NullError,
    })
}

/// Convert a wide cstring to a string
pub(crate) fn widestring_to_string(
    wide_string: &widestring::WideCString,
) -> CubeProgrammerResult<String> {
    wide_string
        .to_string()
        .map_err(|x| CubeProgrammerError::TypeConversion {
            message: format!("Cannot convert widestring to string: {:?}", x),
            source: TypeConversionError::Utf16Error,
        })
}

/// Convert a c_char slice to a null-terminated string
pub(crate) fn c_char_slice_to_string(slice: &[c_char]) -> CubeProgrammerResult<&str> {
    str::from_utf8(bytemuck::cast_slice(slice)).map_err(|x| CubeProgrammerError::TypeConversion {
        message: format!("Failed to convert c_char slice to string: {:?}", x),
        source: TypeConversionError::Utf8Error,
    })
}

/// Convert a string to a cstring
pub(crate) fn string_to_cstring(s: &str) -> CubeProgrammerResult<CString> {
    CString::new(s).map_err(|x| CubeProgrammerError::TypeConversion {
        message: format!("Failed to convert str to cstring: {:?}", x),
        source: TypeConversionError::NullError,
    })
}
