# stm32cubeprogrammer-cli

This CLI aims to provide a simple interface for setting up stm32 targets.
## Supported commands
- Flashing bin and hex files
- Updating BLE stack
- Resetting target
- Mass erasing target
- Enabling read protection
- Disabling read protection
- Resetting target

All commands above can be combined in a single command line invocation by chaining them.
If you need other commands, feel free to open an issue or a pull request. :smile:

## Example usage:
Where `installation_dir` is the path to the root directory of the STM32CubeProgrammer installation.
E.g. `C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer`

```sh
stm32cubeprogrammer-cli --stm32-cube-programmer-dir `installation_dir` reset
```

You can also pass the directory to the STM32CubeProgrammer installation using the `STM32_CUBE_PROGRAMMER_DIR` environment variable.
```sh
STM32_CUBE_PROGRAMMER_DIR=`installation_dir` stm32cubeprogrammer-cli reset
```
You can chain multiple commands together.
```sh
STM32_CUBE_PROGRAMMER_DIR=`installation_dir` stm32cubeprogrammer-cli unprotect reset flash-hex `path_to_hex_file` protect
```

Use the `--list` flag to list available probes.
```sh
stm32cubeprogrammer-cli --stm32-cube-programmer-dir `installation_dir` --list
```

Use `--help` to see all supported commands and options (or see [`crate::parse::Options`])
```sh
stm32cubeprogrammer-cli --help
```
## CLI output
The CLI outputs a JSON object (see [`crate::output::Output`]) which contains information about the selected probe, general information about the target, and the output of each command.
The output is printed to stdout.

## Requirements
There needs to be a Stm32CubeProgrammer installation on your system. The crates are tested using Stm32CubeProgrammer version 2.18.0.

## Platform support
Windows and Linux are supported and tested.

## Warranty
This crate is supplied as is without any warranty. Use at your own risk.

License: MIT
