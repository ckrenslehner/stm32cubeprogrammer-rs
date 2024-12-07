use crate::LogMessageType;

use std::fmt::Debug;
use std::sync::{Arc, Mutex, OnceLock};

static DISPLAY_CALLBACK_HANDLER: OnceLock<Arc<Mutex<dyn DisplayCallback>>> = OnceLock::new();

/// Trait for display callback
/// A library user can implement this trait to receive log messages and update progress bar
pub trait DisplayCallback: Send + Sync + Debug {
    fn init_progressbar(&self);
    fn log_message(&self, message_type: LogMessageType, message: &str);
    fn update_progressbar(&self, current_number: u64, total_number: u64);
}

pub(crate) fn set_display_callback_handler(handler: Arc<Mutex<dyn DisplayCallback>>) {
    DISPLAY_CALLBACK_HANDLER.set(handler).unwrap();
}

pub(crate) fn get_display_callback_handler() -> Option<&'static Arc<Mutex<dyn DisplayCallback>>> {
    DISPLAY_CALLBACK_HANDLER.get()
}
