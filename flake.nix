# names flake
{
  description = "Guess nationality and gender from first name";

  # Get nightly rust from fenix
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat = {
      # For VS Code use of shell.nix, until Nix Env plugin supports flakes
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  # Just a dev env and CI
  outputs = { self, nixpkgs, ... } @ inputs:
    let
      name = "names";
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system:
        let pkgs = import nixpkgs {
          inherit system;
          overlays = [ inputs.fenix.overlay ];
        }; in f pkgs);

      # Rust nightly toolchain with wasm32 support
      nightlyRustToolchain = pkgs: with pkgs.fenix;
        combine [
          (latest.withComponents [
            "cargo"
            "rustc"
            "rust-src"
            "rustfmt"
            "clippy"
          ])
          targets.wasm32-unknown-unknown.latest.rust-std
        ];
    in
    {
      # Development environment
      devShell = forAllSystems (pkgs:
        pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            # Rust stuff
            (nightlyRustToolchain pkgs)
            rust-analyzer-nightly
            libiconv

            # Other packages
            bacon
            trunk
            nixpkgs-fmt
          ];
        }
      );

      # Basic CI
      checks = forAllSystems
        (pkgs: {
          format = pkgs.runCommand "check-format"
            { buildInputs = [ (nightlyRustToolchain pkgs) pkgs.nixpkgs-fmt ]; }
            ''
              ${pkgs.cargo}/bin/cargo fmt --manifest-path ${./.}/Cargo.toml -- --check
              ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${./.}
              touch $out # success
            '';
        });
    };
}
