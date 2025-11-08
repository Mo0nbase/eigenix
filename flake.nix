{
  description = "Eigenix - Dioxus web app with Axum backend and CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    rust-overlay.url = "github:oxalica/rust-overlay";
    dioxus.url = "github:DioxusLabs/dioxus";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      imports = [
        ./cli/cli.nix
        ./backend/backend.nix
        ./web/web.nix
      ];

      perSystem =
        { pkgs, system, ... }:
        {
          # Import nixpkgs with rust-overlay
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          # Set default package to the web frontend
          packages.default = inputs.self.packages.${system}.eigenix-web;
        };
    };
}
