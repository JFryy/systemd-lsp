# Contributing to systemd-lsp

Contributions are welcome, especially from people who know systemd or Podman Quadlet well. Bug reports with a unit file that triggers a wrong diagnostic are just as useful as code.

## Development setup

Requirements: a stable Rust toolchain, and Go 1.25+ if you work on the documentation generator in `tools/doc-sync/`.

```sh
git clone https://github.com/JFryy/systemd-lsp.git
cd systemd-lsp
cargo build
make test
```

To check a file from the command line, run `cargo run -- path/to/unit.service`. The `examples/` directory contains valid and invalid sample units.

## Before opening a pull request

CI runs these, so run them locally first:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
make test
make gen-docs-check   # only needed if you touched docs/ or tools/doc-sync/
```

- Keep pull requests focused on one change.
- Add a test for any behaviour change. Parser, diagnostics, and formatter tests live next to the code in `src/`; end-to-end tests are in `tests/`.
- Diagnostics should follow what systemd itself accepts. If you are unsure, link the relevant man page section in the PR.

## Project layout

- `src/main.rs`: LSP server and request handlers.
- `src/parser.rs`: unit file parser.
- `src/diagnostics.rs`, `src/completion.rs`, `src/formatting.rs`, `src/definition.rs`, `src/semantic_tokens.rs`: individual LSP features.
- `src/constants.rs`: embedded documentation, directive lists, and valid values for directives.
- `docs/`: documentation and directive lists generated from the systemd and Podman manuals. Do not edit by hand.
- `tools/doc-sync/`: Go tool that generates `docs/`.

## Updating directives and documentation

Directive lists and hover documentation come from the upstream manuals. To refresh them after a systemd or Podman release, run `make gen-docs` and commit the changes under `docs/`.

To add a new section or manual page:

1. Add it to `Pages` in `tools/doc-sync/config.go`, and to `SharedIncludes` if it uses the shared exec, kill, or resource-control directives.
2. Run `make gen-docs`.
3. Embed the generated files in `src/constants.rs`.
4. Run `make test-section-coverage` and `make test-directive-coverage`.

Allowed values for a directive (for example `Type=` or `Restart=`) are maintained by hand in `SystemdConstants::valid_values_for_section` in `src/constants.rs`.

## Reporting bugs

Please use the bug report template and include the systemd-lsp version, your editor, and the smallest unit file that reproduces the problem.
