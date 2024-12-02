#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]
pub mod bindings;
pub use bindings::*;

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