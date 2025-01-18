// use std::str::FromStr;

// use log::{debug, info};

// use stm32cubeprogrammer::{
//     cube_programmer::{ConnectedFusProgrammer, ConnectedProgrammer},
//     probe, CubeProgrammer,
// };

// fn connect(
//     api: &mut CubeProgrammer,
//     connect_param: probe::ConnectionParameters,
// ) -> ConnectedProgrammer {
//     let probes = api.list_available_probes().unwrap();

//     if !probes.is_empty() {
//         info!("Found {} ST-Link probes - Trying to connect", probes.len());
//         info!("Connecting to target via probe: {}", probes[0]);

//         let connected_programmer = api
//             .connect_to_target(&probes[0], &probe::Protocol::Swd, &connect_param)
//             .unwrap();

//         connected_programmer
//     } else {
//         panic!("No ST-Link probes found");
//     }
// }

// fn connect_fus(api: &mut CubeProgrammer) -> ConnectedFusProgrammer {
//     let probes = api.list_available_probes();

//     if !probes.is_empty() {
//         info!("Found {} ST-Link probes - Trying to connect", probes.len());
//         info!("Connecting to target via probe: {}", probes[0]);

//         let connected_programmer = api
//             .connect_to_target_fus(&probes[0], &probe::Protocol::Swd)
//             .unwrap();

//         connected_programmer
//     } else {
//         panic!("No ST-Link probes found");
//     }
// }

// // -- TEST CASES -- //
// #[test_log::test]
// fn connect_twice() {
//     dotenvy::dotenv().unwrap();
//     let api = get_api();
//     let probes = api.list_available_probes();

//     if probes.is_empty() {
//         panic!("No ST-Link probes found");
//     }

//     info!("Found {} ST-Link probes - Trying to connect", probes.len());
//     info!("Connecting to target via probe: {}", probes[0]);

//     let _connected_programmer = api
//         .connect_to_target(
//             &probes[0],
//             &probe::Protocol::Swd,
//             &probe::ConnectionParameters::default(),
//         )
//         .unwrap();

//     // Connect to same probe again -> must not work
//     if let Ok(_) = api.connect_to_target(
//         &probes[0],
//         &probe::Protocol::Swd,
//         &probe::ConnectionParameters::default(),
//     ) {
//         panic!("Connecting to the same probe twice must not work");
//     };
// }

// #[test_log::test]
// fn write_and_read() {
//     let data_bytes = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd_Hello_Cube";

//     #[repr(C, packed)]
//     #[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
//     struct MyData {
//         one: u8,
//         two: u8,
//         three: i16,
//         four: u32,
//         five: [u8; 11],
//     }

//     dotenvy::dotenv().unwrap();
//     let mut api = get_api();
//     let connected_programmer = connect(
//         &mut api,
//         probe::ConnectionParameters {
//             connection_mode: probe::ConnectionMode::UnderReset,
//             ..Default::default()
//         },
//     );

//     let address = stm32cubeprogrammer_sys::SRAM_BASE_ADDRESS + 1024;

//     // Write bytes and read pod
//     connected_programmer
//         .write_memory_bytes(address, data_bytes)
//         .unwrap();
//     connected_programmer
//         .write_memory_bytes(address + std::mem::size_of::<MyData>() as u32, data_bytes)
//         .unwrap();

//     let read = connected_programmer
//         .read_memory::<MyData>(address, 2)
//         .unwrap();

//     dbg!(&read);
//     assert_eq!(bytemuck::bytes_of(&read[0]), data_bytes);
//     assert_eq!(read[0], read[1]);

//     // Read elements separately
//     let curr = connected_programmer.read_memory::<u8>(address, 1).unwrap()[0];
//     assert_eq!(curr, read[0].one);

//     let curr = connected_programmer
//         .read_memory::<u8>(address + 1, 1)
//         .unwrap()[0];
//     assert_eq!(curr, read[0].two);

//     let curr = connected_programmer
//         .read_memory::<i16>(address + 2, 1)
//         .unwrap()[0];
//     let three = read[0].three;
//     assert_eq!(curr, three);

//     let curr = connected_programmer
//         .read_memory::<u32>(address + 4, 1)
//         .unwrap()[0];
//     let four = read[0].four;
//     assert_eq!(curr, four);

//     let curr = connected_programmer
//         .read_memory::<[u8; 11]>(address + 8, 1)
//         .unwrap()[0];
//     let five = read[0].five;
//     assert_eq!(curr, five);

//     let address = stm32cubeprogrammer_sys::SRAM_BASE_ADDRESS + 2048;

//     // Write pod and read bytes
//     let data = vec![read[0]; 2];
//     dbg!(&data);

//     connected_programmer
//         .write_memory::<MyData>(address, &data)
//         .unwrap();

//     let read = connected_programmer
//         .read_memory_bytes(address, std::mem::size_of::<MyData>())
//         .unwrap();
//     assert_eq!(read.as_slice(), data_bytes);

//     let read = connected_programmer
//         .read_memory_bytes(
//             address + std::mem::size_of::<MyData>() as u32,
//             std::mem::size_of::<MyData>(),
//         )
//         .unwrap();
//     assert_eq!(read.as_slice(), data_bytes);
// }

// #[test_log::test]
// fn fus_actions() {
//     dotenvy::dotenv().unwrap();
//     let mut api = get_api();

//     let connected_programmer = connect_fus(&mut api);
//     dbg!(connected_programmer.fus_info());

//     // Delete BLE stack
//     connected_programmer.delete_wireless_stack().unwrap();
//     connected_programmer.disconnect();

//     // Reconnect to read updated FUS information
//     let connected_programmer = connect_fus(&mut api);
//     let fus_info = connected_programmer.fus_info();
//     dbg!(&fus_info);
//     assert_eq!(
//         fus_info.wireless_stack_version,
//         stm32cubeprogrammer::fus::Version::from_str("0.0.0").unwrap()
//     );

//     // Upgrade BLE stack
//     let ble_stack_binary = get_path_from_env_file(ENV_STM32_CUBE_PROGRAMMER_BLE_STACK_PATH);
//     info!("Downloading ble stack file: {:?}", ble_stack_binary);

//     connected_programmer
//         .upgrade_wireless_stack(
//             ble_stack_binary,
//             get_address_from_env_file(STM32_CUBE_PROGRAMMER_BLE_STACK_START_ADDRESS),
//             false,
//             true,
//             true,
//         )
//         .unwrap();

//     connected_programmer.disconnect();
//     let connected_programmer = connect_fus(&mut api);
//     dbg!(connected_programmer.fus_info());
// }

// #[test_log::test]
// fn download_hex_file() {
//     dotenvy::dotenv().unwrap();
//     let mut api = get_api();
//     let connected_programmer = connect(&mut api, probe::ConnectionParameters::default());

//     let hex_file = get_path_from_env_file(ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_HEX_PATH);
//     info!("Downloading hex file: {:?}", hex_file);

//     connected_programmer
//         .download_hex_file(hex_file, false, true)
//         .unwrap();
// }

// #[test_log::test]
// fn download_bin_file() {
//     dotenvy::dotenv().unwrap();
//     let mut api = get_api();
//     let connected_programmer = connect(&mut api, probe::ConnectionParameters::default());

//     let hex_file = get_path_from_env_file(ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_PATH);
//     info!("Downloading hex file: {:?}", hex_file);

//     connected_programmer
//         .download_bin_file(
//             hex_file,
//             get_address_from_env_file(ENV_STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_START_ADDRESS),
//             false,
//             true,
//         )
//         .unwrap();
// }
