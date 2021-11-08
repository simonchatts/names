Nix dependencies are pinned and managed by [niv](https://github.com/nmattia/niv).

There are two ways to do this: use rust from nixpkgs, or the mozilla nightlies.

## Mozilla nightly

The Mozilla nightly enables lots more stuff, eg wasm, and that is what we use
here. Just `niv add mozilla/nixpkgs-mozilla` gets you going, and then the

    pinned-date = { date = "2020-07-12"; channel = "nightly"; };

in `shell.nix` actually selects a version.

### Selecting a good date

The Rust Analyzer VS Code plugin is tightly coupled to the date here. So if you
start seeing errors, check out the page for that in VS Code, and look at "Last
Updated" on the right hand side. If it's more recent than the version here,
update the `shell.nix` date.

If you want to do some validation, then check the date with something like:

    curl https://static.rust-lang.org/dist/2020-07-17/channel-rust-nightly.toml

which dumps a toml file containing things like:

    [pkg.rustc.target.x86_64-unknown-linux-gnu]
    available = true
    url = "https://static.rust-lang.org/dist/2020-07-17/rustc-nightly-x86_64-unknown-linux-gnu.tar.gz"
    hash = "40a97faf23cef39210ad23d956fee6c245b90b2d3d8554d6656806f4bc7f1fe0"
    xz_url = "https://static.rust-lang.org/dist/2020-07-17/rustc-nightly-x86_64-unknown-linux-gnu.tar.xz"
    xz_hash = "5f64ff295ccc8cc1aef9e52b9656b3588af8c06c9b65186b0777ea244b7e988e"

for something that successfully built, or:

    [pkg.rustfmt-preview.target.x86_64-unknown-linux-gnu]
    available = false

for something that didn't. Choose a recent enough release that has everything
you need.

## After updating the date

Three things to do after.

One just applies to the first machine to do this, before committing to the repo:
push to cachix, to only do the work once.

The other two apply to all development machines:

 - Trash your `target` directory (at least if you get "incompatible compiler" errors)

 - If you use the [Firefox/Chrome Rust Search
extension](https://rust.extension.sh/) and point it at the local cache of `std`
documentation, then you should also update this after the nix-shell environment
is updated, with the URL from:

    echo file://$(which rustc | sed s:bin/rustc::)/share/doc/rust/html/

## Nixpkgs

If you need to update nixpkgs, then:

 - Check out [nixpkgs](https://github.com/NixOS/nixpkgs/tree/master/pkgs/development/compilers/rust) to see what range of commits
   has the Rust version you want.

 - Check out Hydra for [Intel](https://hydra.nixos.org/jobset/nixos/release-21.05/evals) and
   [ARM](https://hydra.nixos.org/jobset/nixos/release-21.05-aarch64/evals) and pick a sufficiently recent release that looks good.

 - Get the "revision" from the "Inputs" tab on Hydra (eg `38bfbd5d6fed5d89d5a95a494443e82f1d14e07a`).

 - `niv update nixpkgs -v 38bfbd5d6fed5d89d5a95a494443e82f1d14e07a`.
