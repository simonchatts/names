let
  # Set up a pinned environment based on rust-nightly, which is required for WASM.
  #
  # niv manages the pinning of nixpkgs and the mozilla overlay, with the exception of:
  pinned-date = { date = "2021-10-20"; channel = "nightly"; };

  sources = import ./nix/sources.nix;
  mozilla-overlay = import sources.nixpkgs-mozilla;
  pkgs = import sources.nixpkgs { overlays = [ mozilla-overlay ]; };
  nightly-rust = (pkgs.rustChannelOf pinned-date).rust.override {
    extensions = [
      "clippy-preview"
      "rust-analyzer-preview"
      "rustfmt-preview"
      "rust-src"
    ];
    targets = [
      "x86_64-apple-darwin"      # Native build on macOS
      "x86_64-unknown-linux-gnu" # Native build on linux
      "wasm32-unknown-unknown"   # WASM cross-build
    ];
  };

in
with pkgs;
mkShell {

  # Build-time dependencies
  nativeBuildInputs = [

    # Other stuff
    bacon
    trunk

    # Nightly Rust toolchain
    nightly-rust

    # Necessary dependencies
    libiconv

  ] ++ lib.optionals stdenv.isx86_64 [
    # Interactive development stuff that doesn't always build on ARM, where
    # we just need a deployment target.
    wasm-bindgen-cli
  ];
}
