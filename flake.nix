# names flake
{
  description = "Guess nationality and gender from first name";
  outputs = { self, nixpkgs, flake-lib, fenix }:
    let
      # Use rust nightly with wasm32 support
      flib = flake-lib.outputs;
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
    flake-lib.url = "git+ssh://git@github.com/simonchatts/flake-lib?ref=main";
    flake-lib.inputs.nixpkgs.follows = "nixpkgs";
  };
}
