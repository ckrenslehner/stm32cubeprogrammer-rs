#[path = "./test_common.rs"]
mod test_common;

#[cfg(feature = "hardware_tests")]
#[test_log::test]
/// Test reading and writing memory on the target using a custom data structure which implements [`bytemuck::Pod`] and [`bytemuck::Zeroable`]
fn read_and_write_memory() {
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

    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
    /// Some custom data structure to test reading and writing memory
    struct MyData {
        byte: u8,
        half_word: u16,
        word: u32,
        text: [u8; 10],
    }

    impl std::fmt::Display for MyData {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let half_word = self.half_word;
            let word = self.word;

            write!(
                f,
                "MyData {{ byte: 0x{:02X}, half_word: 0x{:04X}, word: 0x{:08X}, text: \"{}\" }}",
                self.byte,
                half_word,
                word,
                std::str::from_utf8(&self.text).unwrap()
            )
        }
    }

    let data = MyData {
        byte: 0x01,
        half_word: 0x0203,
        word: 0x04050607,
        text: "Hello_Cube".as_bytes().try_into().unwrap(),
    };

    let address = stm32cubeprogrammer_sys::SRAM_BASE_ADDRESS;

    // Write the data structure to the target memory
    target_programmer
        .write_memory(address, &[data])
        .expect("Failed to write data structure to target memory");

    // Read the data structure from the target memory
    let read_data = target_programmer
        .read_memory::<MyData>(address, 1)
        .expect("Failed to read data structure from target memory");

    assert_eq!(data, read_data[0]);

    log::info!("Read data: {}", read_data[0]);

    // Also compare the expected bytes by reading the memory as bytes
    let read_bytes = target_programmer
        .read_memory::<u8>(address, std::mem::size_of::<MyData>())
        .expect("Failed to read data structure bytes from target memory");

    let expected_bytes = bytemuck::bytes_of(&data);

    log::info!("Read bytes: {}", hex::encode(&read_bytes));
    assert_eq!(
        read_bytes.as_slice(),
        expected_bytes,
        "Read bytes do not match expected bytes. Expected: {}, Read: {}",
        hex::encode(&expected_bytes),
        hex::encode(&read_bytes)
    );

    // Drop also handles the disconnect
}
