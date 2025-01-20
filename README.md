This repository contains a cargo workspace wrapping the Stm32CubeProgrammer API with a rust interface. The API is provides by ST-Microelectronics and is part of the Stm32CubeProgrammer software. When searching for rust crates I stumbled upon [this](https://github.com/wervin/stm32cubeprog-rs) alternative implementation and was inspired to create my own version. The goal of my implementation is to provide a rust wrapper, a higher level interface and a command line interface to the Stm32CubeProgrammer API.

These parts are represented by the following crates:
- `stm32cubeprogrammer-sys`: Bindings to the Stm32CubeProgrammer API (generated with bindgen)
- `stm32cubeprogrammer`: Safe rust interface around the sys crate
- `stm32cubeprogrammer-cli`: Command line interface which uses `stm32cubeprogrammer` as a library

## Requirements
There needs to be a Stm32CubeProgrammer installation on your system. The crates are tested using Stm32CubeProgrammer version 2.18.0.

## Platform support
Windows and Linux are both supported.

## Warranty
This project is provided as is without any warranty.
