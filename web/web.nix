# Flake-parts module for the Dioxus web frontend
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
        targets = [ "wasm32-unknown-unknown" ];
      };

      rustBuildInputs = [
        pkgs.openssl
        pkgs.libiconv
        pkgs.pkg-config
      ]
      ++ lib.optionals pkgs.stdenv.isLinux [
        pkgs.glib
        pkgs.gtk3
        pkgs.libsoup_3
        pkgs.webkitgtk_4_1
      ]
      ++ lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.darwin.apple_sdk.frameworks;
        [
          IOKit
          Carbon
          WebKit
          Security
          Cocoa
        ]
      );

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
    in
    {
      packages.eigenix-web = rustPlatform.buildRustPackage {
        pname = "eigenix-web";
        version = "0.1.0";
        src = lib.cleanSource ./.;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [
          pkgs.pkg-config
          inputs.dioxus.packages.${system}.dioxus-cli
          pkgs.wasm-bindgen-cli
          pkgs.binaryen
          rustToolchain
        ];

        buildInputs = rustBuildInputs;

        buildPhase = ''
          runHook preBuild
          export HOME=$TMPDIR
          export API_HOST=nixlab
          export API_PORT=3000

          # Build the web package with dx bundle
          dx bundle --release
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          mkdir -p $out

          # Copy the bundled output from dx bundle
          # dx uses the package name from Cargo.toml (which is "web")
          if [ -d "target/dx/web/release/web/public" ]; then
            cp -r target/dx/web/release/web/public/* $out/
          else
            echo "ERROR: dx bundle output directory not found"
            ls -la target/dx/ || echo "target/dx does not exist"
            exit 1
          fi
          runHook postInstall
        '';

        doCheck = false;

        meta = with lib; {
          description = "Eigenix Web Frontend";
          license = licenses.mit;
        };
      };

      devShells.eigenix-web = pkgs.mkShell {
        name = "eigenix-web-dev";
        buildInputs = rustBuildInputs;
        nativeBuildInputs = [
          rustToolchain
          inputs.dioxus.packages.${system}.dioxus-cli
        ];
        shellHook = ''
          export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library";
        '';
      };
    };
}
