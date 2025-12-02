{
  inputs = {
    nixpkgs.url = "nixpkgs/release-25.05";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
          config.allowUnfree = true;
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        formatter = pkgs.nixpkgs-fmt;

        devShells.default = pkgs.mkShell (let
          devRustToolchain = rustToolchain.override {
            extensions = ["rust-analyzer" "rust-src"];
          };

          vscode = pkgs.vscode-with-extensions.override {
            vscodeExtensions = with pkgs.vscode-extensions; [
              bbenoist.nix
              vscodevim.vim
              ms-vscode.cpptools
              skellock.just
            ];
          };
        in {
          packages = [
            devRustToolchain
            pkgs.rustup
            pkgs.espup
            pkgs.bacon
            pkgs.cargo-flamegraph

            pkgs.arduino-cli
            pkgs.arduino
            (pkgs.python3.withPackages (python-pkgs: [
              python-pkgs.pyserial
            ]))

            pkgs.espflash

            pkgs.just
            pkgs.jq

            vscode

            pkgs.pkg-config
            pkgs.openssl
          ];

          RUST_SRC_PATH = "${devRustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            export PATH="$HOME/.rustup/toolchains/esp/bin:$PATH"
            export PATH="$HOME/.rustup/toolchains/esp/xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/bin:$PATH"
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
              pkgs.stdenv.cc.cc.lib
              pkgs.zlib
              pkgs.openssl
            ]}";
          '';
        });
      }
    );
}
