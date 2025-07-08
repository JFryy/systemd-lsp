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

        cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        systemd-lsp = pkgs.rustPlatform.buildRustPackage {
          pname = cargo.package.name;
          version = cargo.package.version;
          src = self;
          cargoHash = "sha256-G1cQWOgtx+Bmi05ji9Z4TBj5pnhglNcfLRoq2zSmyK0=";
        };

        runtimeEnv = with pkgs; [
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
