//! Display callback functions for CubeProgrammer API logging

use log::trace;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::utility;

/// Log message type
#[derive(Debug, Default, Clone, Copy, IntoPrimitive, TryFromPrimitive, strum::Display)]
#[repr(i32)]
pub enum LogMessageType {
    Normal,
    Info,
    GreenInfo,
    Title,
    Warning,
    Error,
    Verbosity1,
    Verbosity2,
    Verbosity3,
    GreenInfoNoPopup,
    WarningNoPopup,
    ErrorNoPopup,

    #[default]
    Unknown = -1,
}

/// Verbosity level
#[derive(Debug, Default, Clone, Copy, IntoPrimitive, strum::Display)]
#[repr(i32)]
pub enum Verbosity {
    Level0,
    Level1,
    Level2,
    Level3,

    #[default]
    Unknown = -1,
}

pub(crate) unsafe extern "C" fn display_callback_init_progressbar() {
    log::trace!("Init progress bar");

    // Forward to display handler if there is one
    if let Some(display_handler) = crate::display::get_display_callback_handler() {
        display_handler.init_progressbar();
    }
}

#[cfg(target_os = "unix")]
pub(crate) unsafe extern "C" fn display_callback_log_message(level: i32, message: *const u32) {
    display_callback_log_message_inner(level, &widestring::WideCString::from_ptr_str(message));
}

#[cfg(target_os = "windows")]
pub(crate) unsafe extern "C" fn display_callback_log_message(level: i32, message: *const u16) {
    display_callback_log_message_inner(level, &widestring::WideCString::from_ptr_str(message));
}

pub(crate) unsafe extern "C" fn display_callback_load_bar(
    mut current_number: i32,
    total_number: i32,
) {
    if current_number > total_number {
        current_number = total_number;
    }

    // Forward to display handler if there is one
    if let Some(display_handler) = crate::display::get_display_callback_handler() {
        if current_number < 0 || total_number < 0 {
            return;
        }

        display_handler.update_progressbar(current_number as u64, total_number as u64);
    }

    log::trace!("Update progress bar: {}/{}", current_number, total_number);
}

fn display_callback_log_message_inner(level: i32, message: &widestring::WideCString) {
    let level = LogMessageType::try_from(level).unwrap_or(LogMessageType::default());

    let log_level = match level {
        LogMessageType::Verbosity3 => log::Level::Trace,
        LogMessageType::Verbosity2 => log::Level::Debug,
        LogMessageType::Verbosity1 => log::Level::Info,
        LogMessageType::Normal => log::Level::Info,
        LogMessageType::Info => log::Level::Info,
        LogMessageType::GreenInfo => log::Level::Info,
        LogMessageType::Title => log::Level::Info,
        LogMessageType::Warning => log::Level::Warn,
        LogMessageType::Error => log::Level::Error,
        LogMessageType::GreenInfoNoPopup => log::Level::Info,
        LogMessageType::WarningNoPopup => log::Level::Warn,
        LogMessageType::ErrorNoPopup => log::Level::Error,
        LogMessageType::Unknown => log::Level::Error,
    };

    if log_level != log::Level::Trace {
        trace!(
            "API log - level: {:?}, message: {}",
            level,
            utility::wide_cstring_to_string(message)
        );
    }

    // Forward to display handler if there is one
    if let Some(display_handler) = crate::display::get_display_callback_handler() {
        display_handler.log_message(level, &utility::wide_cstring_to_string(message));
    }

    log::log!(
        log_level,
        "{:?}, {}",
        level,
        utility::wide_cstring_to_string(message)
    );
}
