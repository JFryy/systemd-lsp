# systemd-ls

A Language Server Protocol (LSP) implementation for systemd unit files, providing editing support with syntax highlighting, diagnostics, autocompletion, and documentation.

## Features

### Core Language Server Features

- **Syntax Analysis** - Complete parsing of systemd unit file structure
- **Diagnostics** - Error detection and validation for sections, directives, directive fields and warnings for non-conventional configurations
- **Autocompletion** - Context-aware suggestions for sections and directives
- **Rich Documentation** - Comprehensive hover information and goto definition
- **Code Formatting** - Formatting of unit files

## Installation

### Building from source

```bash
git clone https://github.com/jfryy/systemdls.git
cd systemd-ls
cargo build --release
```

The binary will be available at `target/release/systemd-ls`.

## Usage


### Manual execution

You can run the language server directly, although there is little reason to do so except for debugging purposes. An editor typically starts and stops the server implicitly.

```bash
./target/release/systemd-ls
```

## Architecture
- **Embedded Documentation** - All manual pages built into the binary
- **No External Dependencies** - Single binary with everything included
- **Cross-Platform** - Works on Linux, macOS, and Windows
- **LSP Standard Compliant** - Compatible with all LSP-capable editors


## About
This project was created as a learning exercise to explore Language Server Protocol implementation in Rust. While functional and comprehensive, it serves as both a (hopefully) useful tool for systemd configuration and a reference implementation for LSP development.

## Contributing
Contributions always welcome.

