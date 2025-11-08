# Flake-parts module for the CLI package
{ inputs, ... }:
{
  perSystem =
    {
      config,
      self',
      pkgs,
      lib,
      system,
      ...
    }:
    let
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
          "clippy"
        ];
      };

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
    in
    {
      packages.eigenix-cli = rustPlatform.buildRustPackage {
        pname = "eigenix-cli";
        version = "0.1.0";
        src = lib.cleanSource ./.;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [
          pkgs.pkg-config
          rustToolchain
        ];

        buildInputs = [
          pkgs.openssl
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
        ];

        # Only build the CLI binary
        cargoBuildFlags = [
          "--package"
          "eigenix-cli"
        ];

        doCheck = false;

        meta = with lib; {
          description = "Eigenix CLI";
          license = licenses.mit;
        };
      };
    };
}
