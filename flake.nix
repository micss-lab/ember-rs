{
  inputs = {
    nixpkgs.url = "nixpkgs/release-24.05";

    flake-utils.url = "github:numtide/flake-utils";

    devenv.url = "github:cachix/devenv";

    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    devenv,
    rust-overlay,
    crane,
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
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        sources = {
          cargo = craneLib.cleanCargoSource ./.;
        };

        metadata = {
          cargo = {
            pname = "no-std-framework-rs";
            version = "0.0.0";
          };
        };

        commonArgs = {
          common = {
            strictDeps = true;
          };

          cargo =
            commonArgs.common
            // metadata.cargo
            // {
              src = sources.cargo;

              # Build-time dependencies.
              nativeBuildInputs = [];
              # Linkable build dependencies.
              buildInputs = [];

              cargoVendorDir = craneLib.vendorMultipleCargoDeps {
                inherit (craneLib.findCargoFiles sources.cargo) cargoConfigs;
                cargoLockList = [
                  ./core/Cargo.lock
                  ./examples/Cargo.lock
                  ./tests/Cargo.lock
                  ./bindings/Cargo.lock
                ];
              };

              CARGO_BUILD_TARGET = "riscv32imc-unknown-none-elf";
            };
        };

        artifacts = {
          cargo = craneLib.buildDepsOnly (commonArgs.cargo
            // {
              cargoCheckExtraArgs = "";
              cargoTestCommand = ''
                true
              '';
            });
        };

        buildArgs = {
          cargo =
            commonArgs.cargo
            // {
              cargoArtifacts = artifacts.cargo;
              # Do not run tests during build process.
              doCheck = false;
            };
        };

        builds = {
          cargo = craneLib.buildPackage buildArgs.cargo;
        };
      in rec {
        formatter = pkgs.nixpkgs-fmt;

        checks = {
          # cargo-build = builds.cargo;
          #
          # cargo-clippy = craneLib.cargoClippy (commonArgs.cargo
          #   // {
          #     cargoArtifacts = artifacts.cargo;
          #     cargoClippyExtraArgs = "-- --deny warnings";
          #   });
          #
          # cargo-doc = craneLib.cargoDoc (commonArgs.cargo
          #   // {
          #     cargoArtifacts = artifacts.cargo;
          #   });
          #
          # cargo-fmt = craneLib.cargoFmt (metadata.cargo
          #   // {
          #     src = sources.cargo;
          #   });
          #
          # cargo-toml-fmt = craneLib.taploFmt (metadata.cargo
          #   // {
          #     src = pkgs.lib.sources.sourceFilesBySuffices sources.cargo [".toml"];
          #   });
          #
          # cargo-test = craneLib.cargoNextest (commonArgs.cargo
          #   // {
          #     cargoArtifacts = artifacts.cargo;
          #   });
          #
          # cargo-hack = craneLib.mkCargoDerivation (commonArgs.cargo
          #   // {
          #     cargoArtifacts = artifacts.cargo;
          #
          #     pnameSuffix = "-hack";
          #
          #     buildPhaseCargoCommand = ''
          #       cargo hack check --workspace --locked
          #     '';
          #
          #     nativeBuildInputs =
          #       commonArgs.cargo.nativeBuildInputs
          #       ++ [
          #         pkgs.cargo-hack
          #       ];
          #   });
          #
          # cargo-machete = craneLib.mkCargoDerivation (commonArgs.cargo
          #   // {
          #     cargoArtifacts = artifacts.cargo;
          #
          #     pnameSuffix = "-machete";
          #
          #     buildPhaseCargoCommand = ''
          #       cargo machete --with-metadata
          #     '';
          #
          #     nativeBuildInputs =
          #       commonArgs.cargo.nativeBuildInputs
          #       ++ [
          #         pkgs.cargo-machete
          #       ];
          #   });
        };

        packages = {
          devenv-up = self.devShells.${system}.default.config.procfileScript;
        };

        devShells.default = devenv.lib.mkShell (let
          lib = nixpkgs.lib;

          # Inherit inputs from flake checks (inputsFrom is not available).
          flakeChecksInputs = let
            from = builtins.attrValues checks ++ builtins.attrValues builds;
            extractInputs = name: (lib.subtractLists from (lib.flatten (lib.catAttrs name from)));
          in
            extractInputs "buildInputs"
            ++ extractInputs "nativeBuildInputs"
            ++ extractInputs "propagatedBuildInputs"
            ++ extractInputs "propagatedNativeBuildInputs";

          devRustToolchain = rustToolchain.override {
            extensions = ["rust-analyzer" "rust-src"];
          };
        in {
          inherit inputs pkgs;
          modules = [
            ({...}: {
              name = metadata.cargo.pname;

              cachix = {
                enable = true;
                pull = ["exellentcoin26-no-std-framework-rs"];
              };

              packages =
                [
                  devRustToolchain
                  pkgs.espup
                  pkgs.bacon

                  pkgs.arduino-cli
                  pkgs.arduino
                  (pkgs.python3.withPackages (python-pkgs: [
                    python-pkgs.pyserial
                  ]))

                  pkgs.just
                ]
                ++ flakeChecksInputs;

              env = {
                RUST_SRC_PATH = "${devRustToolchain}/lib/rustlib/src/rust/library";
              };

              enterShell = ''
                export PATH="$PATH:${self}/bin";
              '';
            })
          ];
        });
      }
    );
}
