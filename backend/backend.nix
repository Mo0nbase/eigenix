# Flake-parts module for the Axum backend
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
      packages.eigenix-backend = rustPlatform.buildRustPackage {
        pname = "eigenix-backend";
        version = "0.1.0";
        src = lib.cleanSource ./.;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [
          pkgs.pkg-config
          rustToolchain
          pkgs.gcc
        ];

        buildInputs = [
          pkgs.openssl
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
        ];

        # Only build the backend binary
        cargoBuildFlags = [
          "--package"
          "eigenix-backend"
        ];

        doCheck = false;

        meta = with lib; {
          description = "Eigenix Axum Backend";
          license = licenses.mit;
        };
      };
    };
}
