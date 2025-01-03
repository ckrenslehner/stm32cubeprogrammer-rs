# Workspace wrapping the Stm32CubeProgrammer API

## Description
This workspace wraps the Stm32CubeProgrammer API to provide a rust interface. The API can be found in the directory where the Stm32CubeProgrammer is installed.

Inspired by https://github.com/wervin/stm32cubeprog-rs but using bindgen to generate the bindings.

Crates:
- `stm32cubeprogrammer-sys`: Bindings to the Stm32CubeProgrammer API (generated with bindgen)
- `stm32cubeprogrammer`: Safe rust interface to the Stm32CubeProgrammer API -> WIP
- `stm32cubeprogrammer-cli`: Command line interface to the Stm32CubeProgrammer API -> WIP (not yet existing)

## Usage
The bindings are generated with bindgen when running `cargo build`. The bindings are generated in the `src` directory of the `stm32cubeprogrammer-sys` crate. The header file `STM32_Programmer_API.h` and the whole `include` directory were copied from the `api` folder of the Stm32CubeProgrammer installation directory. This is tested with Stm32CubeProgrammer version 2.18.0.

### Running the tests
The tests show how to use this crate. You can generate an instance of `CubeProgrammer` via its builder. The builder requires the path to the Stm32CubeProgrammer installation directory. The path needs to be the root of the installation directory, e.g. `C:\STMicroelectronics\STM32Cube\STM32CubeProgrammer`.

You need to add a `.env` file in the root to supply the necessary environment variables. Take a look at the `tests` directory for an example.

A convenient way to run the tests is via `just`:
`just test`

The justfile needs `just` and `nu` to be installed.

## Status
Working functionality:
- Write hex/bin file to target
- Save memory to file
- Mass erase memory
- Enable readout protection
- Disable readout protection
- Read memory
  - As bytes, half words, words
  - As struct (needs to implement bytemuck::Pod + bytemuck::Zeroable)
- Firmware Update Service (only STM32WB5x/35xx)
  - Read installed versions of FUS and BLE stack
  - Delete BLE stack firmware
  - Upgrade BLE stack firmware
  
Functionality to be added:
- There are still many functions in `STM32CubeProgrammer_API.chm` which are not yet implemented.
  -  PRs are more than welcome! 😊

## Platform support
- Windows: Tested
- Linux: Not tested yet but should work

## Warranty
This project is provided as is without any warranty.
