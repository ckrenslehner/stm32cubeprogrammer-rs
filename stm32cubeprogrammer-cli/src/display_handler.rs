use std::borrow::Cow;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;

/// Display handler which wraps a progress bar and a logger
#[derive(Debug)]
pub struct DisplayHandler {
    _multi: MultiProgress,
    progress_bar: indicatif::ProgressBar,
    message: Cow<'static, str>,
}

impl DisplayHandler {
    /// Create a new display handler which combines a progress bar and a logger
    pub fn new(logger: env_logger::Logger) -> Self {
        let multi = MultiProgress::new();

        // Installs new global logger
        LogWrapper::new(multi.clone(), logger).try_init().unwrap();

        let progress_bar = multi.add(ProgressBar::new(0));
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg} - {percent}% - [{wide_bar:.cyan/blue}]")
                .unwrap()
                .progress_chars("#>-"),
        );

        Self {
            _multi: multi,
            progress_bar,
            message: Cow::Owned(String::new()),
        }
    }

    pub fn set_message(&mut self, message: impl Into<Cow<'static, str>>) {
        self.message = message.into();
    }

    pub fn set_finish(&self) {
        self.progress_bar.finish_and_clear();
    }
}

/// Implement the display callback trait for the display handler
/// This allows showing progress and log messages from CubeProgrammer API
impl stm32cubeprogrammer::DisplayCallback for DisplayHandler {
    fn init_progressbar(&self) {
        self.progress_bar.set_message(self.message.clone());
        self.progress_bar.set_length(0);
        self.progress_bar.set_position(0);
    }

    fn log_message(&self, message_type: stm32cubeprogrammer::LogMessageType, message: &str) {
        if message.is_empty() || message == "\n" || message == "\r\n" {
            return;
        }

        match message_type {
            stm32cubeprogrammer::LogMessageType::Normal => log::info!("{}", message),
            stm32cubeprogrammer::LogMessageType::Info => log::info!("{}", message),
            stm32cubeprogrammer::LogMessageType::GreenInfo => log::info!("{}", message),
            stm32cubeprogrammer::LogMessageType::GreenInfoNoPopup => log::info!("{}", message),

            stm32cubeprogrammer::LogMessageType::Warning => log::warn!("{}", message),
            stm32cubeprogrammer::LogMessageType::WarningNoPopup => log::warn!("{}", message),

            stm32cubeprogrammer::LogMessageType::Error => log::error!("{}", message),
            stm32cubeprogrammer::LogMessageType::ErrorNoPopup => log::error!("{}", message),

            stm32cubeprogrammer::LogMessageType::Verbosity1 => log::info!("{}", message),
            stm32cubeprogrammer::LogMessageType::Verbosity2 => log::debug!("{}", message),
            stm32cubeprogrammer::LogMessageType::Verbosity3 => log::trace!("{}", message),

            _ => {}
        }
    }

    fn update_progressbar(&self, current_number: u64, total_number: u64) {
        if current_number == total_number {
            self.progress_bar.finish();
            return;
        }

        if let Some(current_length) = self.progress_bar.length() {
            if current_length != total_number {
                self.progress_bar.set_length(total_number);
            }
        }

        self.progress_bar.set_position(current_number);
    }
}
