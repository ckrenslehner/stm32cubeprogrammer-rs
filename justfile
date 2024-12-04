# Nushell needs to be installeds
set shell := ["nu", "-c"]

# List all the recipes
default:
    just -l

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