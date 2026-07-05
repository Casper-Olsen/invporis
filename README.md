# invporis

![Rust CI](https://github.com/Casper-Olsen/invporis/actions/workflows/rust.yml/badge.svg)

> Work in progress — early development stage

*Pronounced: in-VOR-iss*

## Overview

**invporis** is an investment portfolio tool.

## Requirements

- Rust (latest stable)
- Cargo

## Build and run

Run in development mode:

```bash
cargo run
```

Build a release binary:

```bash
cargo build --release
```

Run the compiled binary:

```bash
./target/release/invporis
```

## Data storage

invporis stores a SQLite database in the system data directory:

- Linux: `~/.local/share/invporis/`
- macOS: `~/Library/Application Support/io.casperolsen.invporis/`
- Windows: `%APPDATA%\casperolsen\invporis\`
