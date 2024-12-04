use std::env;

use assert_cmd::Command;

#[test_log::test]
fn write_hex() {
    dotenvy::dotenv().unwrap();
    let hex_file = env::var("STM32_CUBE_PROGRAMMER_DOWNLOAD_HEX_PATH").unwrap();

    let mut cmd = Command::cargo_bin("stm32cubeprogrammer-cli").unwrap();
    cmd.arg("download-hex").arg(hex_file).assert().success();
}
