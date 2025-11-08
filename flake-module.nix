# Flake-parts module for integrating eigenix into parent flakes
# This imports the individual build modules for CLI, backend, and web
# The parent flake must provide these inputs:
#   - nixpkgs (nixpkgs-unstable recommended)
#   - rust-overlay
#   - dioxus
{ ... }:
{
  imports = [
    ./cli/cli.nix
    ./backend/backend.nix
    ./web/web.nix
  ];
}
