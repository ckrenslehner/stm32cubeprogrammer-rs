# stm32cubeprogrammer

This crate provides a high-level rust API for the STM32CubeProgrammer DLL.

## Example usage:
```rust
use stm32cubeprogrammer::{
    probe::{ConnectionParameters, Protocol, ResetMode},
    CubeProgrammer,
};

// You need to supply the path to the root directory of the STM32CubeProgrammer installation
let programmer = CubeProgrammer::builder()
    .cube_programmer_dir(
        &"C:\\Program Files\\STMicroelectronics\\STM32Cube\\STM32CubeProgrammer",
    )
    .build()
    .expect("Failed to create CubeProgrammer");

let probes = programmer
    .list_available_probes()
    .expect("Failed to list available probes");

probes.iter().for_each(|probe| {
    println!("{:?}", probe);
});

let connected = programmer
    .connect_to_target(&probes[0], &Protocol::Swd, &ConnectionParameters::default())
    .expect("Failed to connect to target");

println!("Target information: {}", connected.target_information());

// If there are multiple connected probes with a target, you can establish multiple connections simultaneously
let connected_other = programmer
    .connect_to_target(&probes[1], &Protocol::Swd, &ConnectionParameters::default())
    .expect("Failed to connect to target");

println!("Other target information: {}", connected_other.target_information());

connected
    .reset_target(ResetMode::Hardware)
    .expect("Failed to reset target");

// Drop also handles the disconnect, but you can also disconnect explicitly
connected.disconnect();

// To update the BLE stack of a stm32wb55xx, you need to connect to the FUS
let connected = programmer
    .connect_to_target_fus(&probes[0], &Protocol::Swd)
    .expect("Failed to connect to FUS. Is the target a stm32wb55xx?");

println!("FUS information: {}", connected.fus_info());
```
More examples can be found in the `tests` directory.

## Supported features:
- Downloading files as hex or bin
- Reading and writing memory
    - Uses the [`bytemuck::Pod`](https://docs.rs/bytemuck/1.21.0/bytemuck/trait.Pod.html) trait for reading and writing data from/to memory
- Resetting the target
- Enabling and disabling readout protection (Level B)
- Reset target
- Mass erase
- FUS operations (only for stm32wb55xx)
- Log messages of the CubeProgrammer DLL are forwarded via the [`display::DisplayCallback`] trait

If there is a feature missing, feel free to open an issue or a pull request. :smile:

## Running the tests
The tests require a STM32CubeProgrammer installation on the host machine and a connected st-link probe with a target.
To run the tests add a `.env` which at least contains the path to the STM32CubeProgrammer installation directory:
```env
STM32_CUBE_PROGRAMMER_DIR="C:\\Program Files\\STMicroelectronics\\STM32Cube\\STM32CubeProgrammer"
```
A list of the expected environment variables can be found in the `test_common.rs` file.

## Other crates similar to this one
When I was looking for a rust API for the STM32CubeProgrammer DLL, I found [this](https://github.com/wervin/stm32cubeprog-rs) crate.
Playing around with it, I got interested in the topic and decided to try writing my own version. :rocket:

## Requirements
There needs to be a Stm32CubeProgrammer installation on your system. The crates are tested using Stm32CubeProgrammer version 2.18.0.

## Platform support
Windows and Linux are supported and tested.

## Warranty
This crate is supplied as is without any warranty. Use at your own risk.

License: MIT
