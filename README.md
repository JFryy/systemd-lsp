# systemd-lsp

A Language Server Protocol (LSP) implementation for systemd unit files, providing editing support with syntax highlighting, diagnostics, autocompletion, and documentation.

## Features

![Demo](examples/demo.gif)

### Core Language Server Features

- **Syntax Analysis** - Complete parsing of systemd unit file structure
- **Diagnostics** - Error detection and validation for sections, directives, directive fields and warnings for non-conventional configurations
- **Autocompletion** - Context-aware suggestions for sections and directives
- **Rich Documentation** - Comprehensive hover information and goto definition
- **Code Formatting** - Formatting of unit files

## Installation

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))

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
vim.api.nvim_create_autocmd("BufEnter", {
    pattern = "*.service",
    callback = function()
        vim.bo.filetype = "systemd"

        local configs = require('lspconfig.configs')
        if not configs.systemd_lsp then
            configs.systemd_lsp = {
                default_config = {
                    cmd = { '/path/to/systemd-lsp/target/release/systemd-lsp' },
                    filetypes = { 'systemd' },
                    root_dir = require('lspconfig.util').find_git_root,
                },
            }
        end

        require('lspconfig').systemd_lsp.setup({
            autostart = true,
            single_file_support = true,
        })
    end,
})
```

Replace `/path/to/systemd-lsp/target/release/systemd-lsp` with the actual path to your built binary.

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

## Contributing
Contributions always welcome.

