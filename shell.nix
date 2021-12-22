# Backwards-compatible shell.nix from flake devShell
let
  lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  flake-compat =
    fetchTarball {
      url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
      sha256 = lock.nodes.flake-compat.locked.narHash;
    };
  compat = import flake-compat { src = ./.; };
in
compat.shellNix
