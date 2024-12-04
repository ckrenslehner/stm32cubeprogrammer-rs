use std::ffi::{c_char, CStr, CString};
use std::path::Path;

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

/// Convert a path to a CString
/// If the path is a extended length path, the prefix will be removed
pub(crate) fn path_to_cstring(path: impl AsRef<Path>) -> CString {
    let path = path
        .as_ref()
        .to_str()
        .expect("Cannot convert path to string");

    CString::new(remove_extended_path_prefix(path)).expect("Cannot convert path to CString")
}

/// Convert a path to a CString
/// If the path is a extended length path, the prefix will be removed
pub(crate) fn path_to_wide_cstring(path: impl AsRef<Path>) -> widestring::WideCString {
    let path = path
        .as_ref()
        .to_str()
        .expect("Cannot convert path to string");

    widestring::WideCString::from_str(remove_extended_path_prefix(path))
        .expect("Cannot convert path to WideCString")
}

/// Convert a wide cstring to a string
pub(crate) fn wide_cstring_to_string(wide_cstring: &widestring::WideCString) -> String {
    wide_cstring
        .to_string()
        .expect("Cannot convert WideCString to string")
}

/// Convert a c_char slice to a null-terminated string
pub(crate) fn cchar_to_null_terminated_string(slice: &[c_char]) -> &str {
    let cstr =
        CStr::from_bytes_until_nul(bytemuck::cast_slice(slice)).expect("Failed to convert CStr");
    cstr.to_str().expect("Failed to convert CStr to str")
}
