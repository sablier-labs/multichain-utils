# List available commands when just is called without arguments
default:
    just --list

# Build the project
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean 

# Install collect-fee
install-collect-fee:
    cargo install --path crates/collect-fee

# Install deployer
install-deployer:
    cargo install --path crates/deployer

# Format code
fmt:
    cargo +nightly fmt --all

# Delegate to a crate’s justfile
just-collect-fee *ARGS:
    just --justfile crates/collect-fee/Justfile {{ARGS}}

just-deployer *ARGS:
    just --justfile crates/deployer/Justfile {{ARGS}}

# Run clippy lints across workspace
lint:
    cargo clippy --workspace --all-targets --all-features

# Run tests
test:
    cargo test