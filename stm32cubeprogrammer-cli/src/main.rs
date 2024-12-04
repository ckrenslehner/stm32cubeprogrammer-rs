use std::env;

use anyhow::{Context, Ok};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use stm32cubeprogrammer::CubeProgrammer;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download a hex file to a device
    DownloadHex {
        hex_file: std::path::PathBuf,
        #[arg(long, default_value_t = false)]
        erase: bool,
        #[arg(long, default_value_t = false)]
        skip_verify: bool,
    },
}

/// Display handler which wraps a progress bar and a logger
struct DisplayHandler {
    progress_bar: indicatif::ProgressBar,
}

impl DisplayHandler {
    fn new(logger: env_logger::Logger) -> Self {
        let multi = MultiProgress::new();

        LogWrapper::new(multi.clone(), logger).try_init().unwrap();

        let progress_bar = ProgressBar::new(0);
        progress_bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"));

        let progress_bar = multi.add(progress_bar);

        Self { progress_bar }
    }
}

impl stm32cubeprogrammer::DisplayCallback for DisplayHandler {
    fn init_progressbar(&self) {
        self.progress_bar.set_length(0);
        self.progress_bar.set_position(0);
    }

    fn log_message(&self, message_type: stm32cubeprogrammer::LogMessageType, message: &str) {
        log::trace!("{}: {}", message_type, message);
    }

    fn update_progressbar(&self, current_number: u64, total_number: u64) {
        if current_number == total_number {
            self.progress_bar.finish();
            return;
        }

        self.progress_bar.set_length(total_number);
        self.progress_bar.set_position(current_number);
    }
}

/// Sample CLI application for CubeProgrammer. Needs .env file with STM32_CUBE_PROGRAMMER_DIR set or STM32_CUBE_PROGRAMMER_DIR environment variable set
fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let logger = env_logger::Builder::new()
        .filter_level(cli.verbosity.into())
        .build();

    if dotenvy::dotenv().is_err() {
        log::warn!("No .env file found");
    }

    let cube_programmer_dir = env::var("STM32_CUBE_PROGRAMMER_DIR")
        .with_context(|| "STM32_CUBE_PROGRAMMER_DIR environment variable not set")?;

    let programmer = CubeProgrammer::builder()
        .cube_programmer_dir(cube_programmer_dir)
        .display_callback(std::sync::Arc::new(DisplayHandler::new(logger)))
        .build()
        .with_context(|| "Failed to create CubeProgrammer instance")?;

    match &cli.command {
        Commands::DownloadHex {
            hex_file,
            erase,
            skip_verify,
        } => {
            println!("## Download Hex File");
            println!("#");
            println!("# Path: {:?}", hex_file);
            println!("# Skip erase: {}, Verify: {}", !erase, !skip_verify);
            println!("##");

            let probes = programmer.list_connected_st_link_probes();
            if probes.is_empty() {
                log::warn!("No probes found");
                return Ok(());
            }

            println!("Found {} probes", probes.len());
            println!("Using first probe: {}", probes[0]);

            let connected = programmer.connect_to_target(&probes[0])?;
            connected
                .download_hex_file(hex_file, !*erase, *skip_verify)
                .with_context(|| "Failed to download hex file")?;

            Ok(())
        }
    }
}
