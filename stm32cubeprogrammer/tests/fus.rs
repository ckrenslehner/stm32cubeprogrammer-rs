use std::str::FromStr;

#[path = "./test_common.rs"]
mod test_common;

#[test_log::test]
/// Test upgrading the BLE stack
fn update_ble_stack() {
    let programmer = test_common::init_programmer();
    let fus_programmer =
        test_common::connect_to_target_fus(&programmer, &stm32cubeprogrammer::probe::Protocol::Swd);

    let ble_stack_binary = test_common::EnvVar::BleStackPath.get();
    let ble_stack_address = stm32cubeprogrammer::utility::HexAddress::from_str(
        &test_common::EnvVar::BleStackStartAddress
            .get()
            .to_string_lossy(),
    )
    .unwrap();

    log::info!(
        "Updating BLE stack - binary: {:?} ; address: 0x{:x}",
        ble_stack_binary,
        ble_stack_address.0
    );

    dbg!(fus_programmer.fus_info());

    fus_programmer
        .upgrade_wireless_stack(ble_stack_binary, ble_stack_address.0, false, true, true)
        .unwrap();

    fus_programmer.disconnect();

    // Reconnect to check if the update was successful
    let fus_programmer =
        test_common::connect_to_target_fus(&programmer, &stm32cubeprogrammer::probe::Protocol::Swd);

    dbg!(fus_programmer.fus_info());

    fus_programmer.start_wireless_stack().unwrap();

    // Drop also handles the disconnect
}
