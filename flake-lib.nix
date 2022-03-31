# Library flake, providing common functions for handling other flakes
{
  outputs = { self, nixpkgs }: {
    lib = rec {
      # We care only about 64-bit platforms
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      # Create a `forAllSystems` function including the specified overlays
      forAllSystemsWith = overlays: f:
        let useSystem = system:
          let pkgs = import nixpkgs { inherit overlays system; };
          in f pkgs;
        in
        nixpkgs.lib.genAttrs systems useSystem;

      # And no-overlay version
      forAllSystems = forAllSystemsWith [ ];

      ################################################################

      # Rust toolchain options, assuming fenix overlay is applied:
      #
      #  - Stable (but with nightly rust-analyzer)
      #  - Nightly
      #  - Nightly with wasm32 cross-compilation support
      rustComponents = [ "cargo" "rustc" "rust-src" "rustfmt" "clippy" ];
      stableRust = pkgs: with pkgs.fenix;
        combine [
          (stable.withComponents rustComponents)
          pkgs.rust-analyzer-nightly
        ];
      nightlyRust = pkgs: with pkgs.fenix;
        combine [
          (latest.withComponents rustComponents)
          pkgs.rust-analyzer-nightly
        ];
      nightlyRustWithWasm = pkgs: with pkgs.fenix;
        combine [
          (latest.withComponents rustComponents)
          pkgs.rust-analyzer-nightly
          targets.wasm32-unknown-unknown.latest.rust-std
        ];

      # Random helpers
      readCargoToml = path:
        let file = builtins.readFile (path + "/Cargo.toml");
        in (builtins.fromTOML file).package;

      # Just include Cargo.toml, Cargo.lock, src/**, *.[18]
      rustFilterSource =
        let regex = ".*/Cargo\.(lock|toml)|.*/src($|/.*)|.*\.[1-8]";
        in
        builtins.filterSource (path: _: builtins.match regex path != null);

      ################################################################

      # Basic CI checks

      # Rust
      checkRustFormatWith = rustToolchain: path: pkgs:
        pkgs.runCommand "check-format-for-rust-and-nix"
          { buildInputs = [ (rustToolchain pkgs) pkgs.nixpkgs-fmt ]; }
          ''
            cargo fmt --manifest-path ${path}/Cargo.toml -- --check
            nixpkgs-fmt --check ${./.}
            touch $out # success
          '';

      checkRustFormat = path: pkgs:
        pkgs.runCommand "check-format-for-rust-and-nix"
          { buildInputs = [ pkgs.cargo pkgs.rustfmt pkgs.nixpkgs-fmt ]; }
          ''
            cargo fmt --manifest-path ${path}/Cargo.toml -- --check
            nixpkgs-fmt --check ${./.}
            touch $out # success
          '';

      # Python
      checkPythonFormatting = path: pkgs:
        pkgs.runCommand "check-format-for-python-and-nix"
          { buildInputs = [ pkgs.black pkgs.nixpkgs-fmt ]; }
          ''
            black --check ${path}
            nixpkgs-fmt --check ${./.}
            touch $out # success
          '';


      ################################################################

      # Back to admin - basic dev env and CI for this flake itself
      devShell = forAllSystems (pkgs:
        pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            nixpkgs-fmt
          ];
        }
      );
      checks = forAllSystems (pkgs: {
        format = pkgs.runCommand "check-format"
          { buildInputs = [ pkgs.nixpkgs-fmt ]; }
          ''
            ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${./.}
            touch $out
          '';
      });
    };
  };
}
