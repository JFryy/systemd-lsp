# systemd-ls

A Language Server Protocol (LSP) implementation for systemd unit files written in Rust.

This project was created as a learning exercise to understand how Language Server Protocol works and to provide better tooling support for systemd configuration files.

## Features

- **Comprehensive Documentation** - Full systemd man page documentation on hover
- **Smart Completions** - Context-aware completions for sections, directives, and values  
- **Security-Focused** - Extensive support for systemd security directives
- **Real-time Validation** - Immediate feedback on syntax and configuration errors
- **Rich Hover Support** - Detailed explanations with examples and recommendations

## Supported systemd unit types

- Service units (`.service`)
- Socket units (`.socket`)
- Timer units (`.timer`)
- Path units (`.path`)
- Mount units (`.mount`)
- Automount units (`.automount`)
- Swap units (`.swap`)
- Target units (`.target`)
- Device units (`.device`)
- Slice units (`.slice`)
- Scope units (`.scope`)

## Installation

### Building from source

```bash
git clone https://github.com/jfryy/systemd-ls.git
cd systemd-ls
cargo build --release
```

The binary will be available at `target/release/systemd-ls`.

## Usage

### With your editor

Configure your LSP-compatible editor to use `systemd-ls` for systemd unit files.

#### VS Code

Add to your `settings.json`:

```json
{
  "files.associations": {
    "*.service": "systemd",
    "*.socket": "systemd",
    "*.timer": "systemd",
    "*.path": "systemd",
    "*.mount": "systemd",
    "*.automount": "systemd",
    "*.swap": "systemd",
    "*.target": "systemd",
    "*.device": "systemd",
    "*.slice": "systemd",
    "*.scope": "systemd"
  }
}
```

#### Neovim

```lua
-- Add to your init.lua
local configs = require('lspconfig.configs')

if not configs.systemd_ls then
  configs.systemd_ls = {
    default_config = {
      cmd = { 'systemd-ls' }, -- assumes systemd-ls is in PATH
      filetypes = { 'systemd' },
      root_dir = function(fname)
        return require('lspconfig.util').find_git_ancestor(fname) or vim.fn.getcwd()
      end,
    },
  }
end

require('lspconfig').systemd_ls.setup {}

-- File type detection
vim.filetype.add({
  extension = {
    service = 'systemd',
    socket = 'systemd',
    target = 'systemd',
    timer = 'systemd',
    mount = 'systemd',
    automount = 'systemd',
    swap = 'systemd',
    path = 'systemd',
  }
})
```

### Manual execution

You can run the language server directly:

```bash
./target/release/systemd-ls
```

The server communicates via stdin/stdout using the LSP protocol.

## Supported LSP Features

- [x] `textDocument/diagnostic` - Error checking and validation
- [x] `textDocument/completion` - Code completion
- [x] `textDocument/hover` - Documentation on hover
- [x] `textDocument/didOpen` - File opened
- [x] `textDocument/didChange` - File changed
- [x] `textDocument/didSave` - File saved
- [x] `textDocument/didClose` - File closed

## Validation Rules

### Sections

The language server validates that sections are recognized systemd sections:
- `[Unit]`, `[Service]`, `[Socket]`, `[Timer]`, `[Path]`, `[Mount]`, `[Automount]`
- `[Swap]`, `[Target]`, `[Device]`, `[Slice]`, `[Scope]`, `[Install]`

### Directives

Each section has a set of valid directives. The language server will warn about:
- Unknown directives in sections
- Invalid values for specific directives (e.g., `Type=` in `[Service]` section)

### Service-specific validation

- `Type=` values: `simple`, `exec`, `forking`, `oneshot`, `dbus`, `notify`, `idle`
- `Restart=` values: `no`, `on-success`, `on-failure`, `on-abnormal`, `on-watchdog`, `on-abort`, `always`
- `ExecStart=` cannot be empty

## About

This project was created as a learning exercise to explore Language Server Protocol implementation in Rust. While functional and comprehensive, it serves as both a useful tool for systemd configuration and a reference implementation for LSP development.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Author

James Fotherby (fotherby1@gmail.com)