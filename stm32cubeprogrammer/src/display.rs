use crate::LogMessageType;
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

lazy_static! {
    static ref DISPLAY_CALLBACK_HANDLER: RwLock<Option<Arc<dyn DisplayCallback>>> =
        RwLock::new(None);
}

/// Trait for display callback
/// A library user can implement this trait to receive log messages and update progress bar
pub trait DisplayCallback: Send + Sync {
    fn init_progressbar(&self);
    fn log_message(&self, message_type: LogMessageType, message: &str);
    fn update_progressbar(&self, current_number: u64, total_number: u64);
}

pub(crate) fn set_display_callback_handler(handler: Arc<dyn DisplayCallback>) {
    let mut lock = DISPLAY_CALLBACK_HANDLER.write().unwrap();
    *lock = Some(handler);
}

pub(crate) fn get_display_callback_handler() -> Option<Arc<dyn DisplayCallback>> {
    let lock = DISPLAY_CALLBACK_HANDLER.read().unwrap();
    lock.clone()
}