#![cfg(feature = "hardware_tests")]

use std::assert_eq;
use stm32cubeprogrammer::CoreRegister;

#[path = "./test_common.rs"]
mod test_common;

#[test_log::test]
/// Test reading and writing memory on the target using a custom data structure which implements [`bytemuck::Pod`] and [`bytemuck::Zeroable`]
fn read_and_write_core_register() {
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

    let value = 0x12345678;
    target_programmer
        .write_core_register(CoreRegister::R0, value)
        .expect("Failed to write core register");
    let r0 = target_programmer
        .read_core_register(CoreRegister::R0)
        .expect("Failed to read core register");
    println!("R0: 0x{:08X}", r0);

    assert_eq!(r0, value);

    // Drop also handles the disconnect
}
