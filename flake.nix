{
  inputs = {
    nixpkgs.url = "nixpkgs/release-24.05";

    flake-utils.url = "github:numtide/flake-utils";

    devenv.url = "github:cachix/devenv";

    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    devenv,
    rust-overlay,
    ...
  } @ inputs:
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

        packages = {
          devenv-up = self.devShells.${system}.default.config.procfileScript;
        };

        devShells.default = devenv.lib.mkShell (let
          devRustToolchain = rustToolchain.override {
            extensions = ["rust-analyzer" "rust-src"];
          };
        in {
          inherit inputs pkgs;
          modules = [
            ({...}: {
              packages = [
                devRustToolchain
                pkgs.rustup
                pkgs.espup
                pkgs.bacon

                pkgs.arduino-cli
                pkgs.arduino
                (pkgs.python3.withPackages (python-pkgs: [
                  python-pkgs.pyserial
                ]))

                pkgs.just
                pkgs.jq
              ];

              env = {
                RUST_SRC_PATH = "${devRustToolchain}/lib/rustlib/src/rust/library";
                LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
                  pkgs.stdenv.cc.cc.lib
                  pkgs.zlib
                  pkgs.libxml2
                ]}";
              };

              enterShell = ''
                export PATH="$HOME/.rustup/toolchains/esp/bin:$PATH"
                export PATH="$HOME/.rustup/toolchains/esp/xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/bin:$PATH"
              '';
            })
          ];
        });
      }
    );
}
