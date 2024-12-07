# Nushell needs to be installeds
set shell := ["nu", "-c"]

shebang := if os() == 'windows' {
  'nu.exe'
} else {
  '/usr/bin/env nu'
}

# List all the recipes
default:
    just -l

sample-env-file:
    #!{{shebang}}
    let content = 'STM32_CUBE_PROGRAMMER_DIR = "<PATH TO STM32_CUBE_PROGRAMMER ROOT DIR>"
    STM32_CUBE_PROGRAMMER_DOWNLOAD_HEX_PATH = "<PATH TO HEX FILE>"
    STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_PATH = "<PATH TO BIN FILE>"
    STM32_CUBE_PROGRAMMER_DOWNLOAD_BIN_START_ADDRESS = "<START ADDRESS e.g. 0x08000000>"
    STM32_CUBE_PROGRAMMER_BLE_STACK_PATH = "<PATH TO BLE STACK BIN FILE>"
    STM32_CUBE_PROGRAMMER_BLE_STACK_START_ADDRESS = "<START ADDRESS e.g. 0x080CE000>"'

    echo $content | save .env

# Run all tests or a specific test with a specific verbosity
# The log level maps to the `log` crate log levels: trace, debug, info, warn, error
test name="" log_level="trace":
    echo "Running tests..."
    # Add your test commands here
    RUST_LOG={{log_level}} cargo test {{name}} -- --test-threads=1 --nocapture --show-output

# Generate the changelog with git-cliff
changelog:
    git-cliff | save CHANGELOG.md --force

# Release the project
# TODO: Add CI and so on
release: changelog
    cargo clippy