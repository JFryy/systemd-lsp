# systemd-lsp

[![Rust Build](https://github.com/jfryy/systemd-lsp/workflows/Rust/badge.svg)](https://github.com/jfryy/systemd-lsp/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/systemd-lsp.svg)](https://crates.io/crates/systemd-lsp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/systemd-lsp.svg)](https://crates.io/crates/systemd-lsp)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/jfryy/systemd-lsp.svg)](https://github.com/jfryy/systemd-lsp/releases)

A Language Server Protocol (LSP) implementation for systemd unit files, providing editing support with syntax highlighting, diagnostics, autocompletion, and documentation.

## Features

![Demo](examples/demo.gif)

### Core Language Server Features

- **Syntax Analysis** - Complete parsing of systemd unit file structure
- **Context Aware** - Context aware automcompletion for directives for corresponding sections
- **Diagnostics** - Error detection and validation for sections, directives, directive fields and warnings for non-conventional configurations
- **Autocompletion** - Context-aware suggestions for sections and directives
- **Rich Documentation** - Comprehensive hover information and goto definition
- **Code Formatting** - Formatting of unit files

## Installation

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))

### Install via Cargo

```bash
cargo install systemd-lsp
```

The binary will be installed to `~/.cargo/bin/systemd-lsp` (or `%USERPROFILE%\.cargo\bin\systemd-lsp` on Windows).

### Building from source

```bash
git clone https://github.com/jfryy/systemd-lsp.git
cd systemd-lsp
cargo build --release
```

The binary will be available at `target/release/systemd-lsp`.

### Compilation

The project is built using Cargo, Rust's package manager. The `--release` flag optimizes the build for performance. For development, you can use `cargo build` for faster compilation times with debug information.

## Usage

### Neovim

Add this configuration to your Neovim setup:
```lua
-- Automatically set filetype and start LSP for specific systemd unit file patterns
vim.api.nvim_create_autocmd("BufEnter", {
    pattern = { "*.service", "*.mount", "*.device", "*.nspawn", "*.target", "*.timer" },
    callback = function()
        vim.bo.filetype = "systemd"
        vim.lsp.start({
            name = 'systemd_ls',
            cmd = { '/path/to/systemd-lsp' }, -- Update this path to your systemd-lsp binary
            root_dir = vim.fn.getcwd(),
        })
    end,
})
```

### Emacs
```scheme
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(systemd-mode "/path/to/systemd-lsp")))
```

Replace `/path/to/systemd-lsp` with the actual path to your built binary.

### Manual execution

You can run the language server directly, although there is little reason to do so except for debugging purposes. An editor typically starts and stops the server implicitly.

```bash
./target/release/systemd-lsp
```

## Architecture
- **Embedded Documentation** - All manual pages built into the binary
- **No External Dependencies** - Single binary with everything included
- **Cross-Platform** - Works on Linux, macOS, and Windows
- **LSP Standard Compliant** - Compatible with all LSP-capable editors


## About
This project is designed to simplify the editing of Unit files by providing validation, autocompletion, and formatting features commonly available for modern languages and file formats. Inspired by [systemd-language-server](https://github.com/psacawa/systemd-language-server), it offers enhanced functionality and improved performance, leveraging Rust's memory safety and efficiency.
I don't always have a ton of time for staying up to date with all the changes that occur to systemd, so contributions or other maintainers of this language server are greatly appreciated. This language
server has the somewhat unique difficulty of maintaing a large static list of directives for sections that can change on occasion rather than having the challenge of maintaining an AST for an actual programming language. 
Any systemD experts would be appreciated for their input, feedback, or maintaining of this language server while the latter half of bugs are squashed out.

## Contributing
Contributions are always encouraged and welcome. Please provide details and tests if appropriate for your changes.

