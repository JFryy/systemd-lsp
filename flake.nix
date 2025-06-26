{
  description = "Flake for github:JFryy/systemd-lsp";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        systemd-lsp = pkgs.rustPlatform.buildRustPackage {
          pname = "systemd-lsp";
          version = "0.1.0";
          src = self;
          cargoHash = "sha256-bYksgHTXomeEJuSk800+/PYXzMvrixSjfPnoqxStWAA=";
        };

        runtimeEnv = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
        ];
      in
      {
        packages.default = systemd-lsp;

        devShells.default = pkgs.mkShell {
          packages = runtimeEnv ++ [ systemd-lsp ];
          RUST_SRC_PATH = "${pkgs.rustc}/lib/rustlib/src/rust/library";
        };
      }
    );
}
