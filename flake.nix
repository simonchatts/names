# names flake
{
  description = "Guess nationality and gender from first name";
  outputs = { self, nixpkgs, fenix }:
    let
      # Use rust nightly with wasm32 support
      flib =
        # Cheesy using local flake as library (doesn't use it's own self)
        ((import ./flake-lib.nix).outputs { inherit nixpkgs self; }).lib;
      forAllSystems = flib.forAllSystemsWith [ fenix.overlay ];
      rustToolchain = pkgs: flib.nightlyRustWithWasm pkgs;
      checkFormatting = flib.checkRustFormatWith rustToolchain ./.;
    in
    {
      # Development environment
      devShell = forAllSystems (pkgs:
        pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            (rustToolchain pkgs)
            bacon
            trunk
            nixpkgs-fmt
          ];
          buildInputs = with pkgs; lib.optionals stdenv.isDarwin [ libiconv ];
        }
      );

      # Basic CI
      checks = forAllSystems (pkgs: {
        format = checkFormatting pkgs;
      });
    };

  # Inputs: fenix for nightly rust, and flake-lib for boilerplate
  inputs = {
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };
}
