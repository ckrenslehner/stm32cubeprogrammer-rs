#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused, clippy::all)]

#[cfg(windows)]
include!("bindings_windows.rs");

#[cfg(unix)]
include!("bindings_unix.rs");

// Re-export libloading so that the user doesn't have to depend on it
pub use libloading;

#[cfg(windows)]
pub const PATH_API_LIBRARY_RELATIVE: &str = "api/lib/CubeProgrammer_API.dll";

#[cfg(unix)]
pub const PATH_API_LIBRARY_RELATIVE: &str = "lib/libCubeProgrammer_API.so";

#[cfg(windows)]
pub const PATH_LOADER_DIR_RELATIVE: &str = "bin";

#[cfg(unix)]
pub const PATH_LOADER_DIR_RELATIVE: &str = "bin";
