use std::str::FromStr;

#[path = "./test_common.rs"]
mod test_common;

#[cfg(feature = "hardware_tests")]
#[test_log::test]
/// Test flashing a hex file and a bin file to the target
fn download_files() {
    let programmer = test_common::init_programmer();
    let target_programmer = test_common::connect_to_target(
        &programmer,
        &stm32cubeprogrammer::probe::Protocol::Swd,
        &stm32cubeprogrammer::probe::ConnectionParameters {
            // Use under reset mode to halt the target before running any instructions
            // to avoid the RAM being overwritten by the target firmware
            connection_mode: stm32cubeprogrammer::probe::ConnectionMode::UnderReset,
            ..Default::default()
        },
    );

    let hex_file = test_common::EnvVar::DownloadHexPath.get();
    target_programmer
        .download_hex_file(hex_file, false, true)
        .expect("Failed to download hex file");

    target_programmer
        .reset_target(stm32cubeprogrammer::probe::ResetMode::Software)
        .expect("Failed to reset target");

    let bin_file = test_common::EnvVar::DownloadBinPath.get();
    let bin_file_address = stm32cubeprogrammer::utility::HexAddress::from_str(
        &test_common::EnvVar::DownloadBinStartAddress
            .get()
            .to_string_lossy(),
    )
    .expect("Failed to parse bin file address");
    target_programmer
        .download_bin_file(bin_file, bin_file_address.0, false, true)
        .expect("Failed to download bin file");

    // Drop also handles the disconnect
}
