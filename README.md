# Workspace wrapping the Stm32CubeProgrammer API

This workspace wraps the Stm32CubeProgrammer API to provide a rust interface. The API can be found in the directory where the Stm32CubeProgrammer is installed.

Crates:
- `stm32cubeprogrammer-sys`: Bindings to the Stm32CubeProgrammer API (generated with bindgen)
- `stm32cubeprogrammer`: Safe rust interface to the Stm32CubeProgrammer API -> WIP
- `stm32cubeprogrammer-cli`: Command line interface to the Stm32CubeProgrammer API -> WIP (not yet existing)

## Usage
The bindings are generated with bindgen when running `cargo build`. The bindings are generated in the `src` directory of the `stm32cubeprogrammer-sys` crate. The header file `STM32_Programmer_API.h` and the whole `include` directory were copied from the `api` folder of the Stm32CubeProgrammer installation directory. This is tested with Stm32CubeProgrammer version 2.18.0. To create an instance of `CubeProgrammerApi` pass the path to the Stm32CubeProgrammer installation directory to the `new` function. The path needs to be the root of the installation directory, e.g. `C:\STMicroelectronics\STM32Cube\STM32CubeProgrammer`.

```rust

## License
This project is licensed under the MIT license.

## Warranty
This project is provided as is without any warranty.
