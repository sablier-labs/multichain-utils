# Multichain Utils

Rust-based utility scripts meant to be used by the Sablier team. This repo contains the following crates:

- `collect-fee` - A CLI tool for collecting fees from Sablier contracts across multiple chains.
- `deployer` - A CLI tool for deploying Sablier contracts across multiple chains.

## Requirements

Rust and Cargo must be installed on your machine. See the installation guide [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Then, install the following dependencies globally:

- [cargo-watch](https://crates.io/crates/cargo-watch) - A Cargo subcommand for watching file changes
- [just](https://github.com/casey/just) - A command runner for development tasks

Clone this repository

```bash
git clone https://github.com/sablier-labs/multichain-deployer.git
```

## Installation

### List all available commands

```bash
just
```

### Install collect-fee crate:

```bash
just install-collect-fee
```

### Install deployer crate:

```bash
just install-deployer
```

### Lint code

```bash
just lint
```

## Usage

```bash
deployer --help
```

Each crate has its own `justfile` for managing commands. You can run `just` in each crate's directory to list the available commands.
