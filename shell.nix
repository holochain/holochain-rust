let

  pkgs = import ./holonix/nixpkgs/nixpkgs.nix;
  darwin = import ./holonix/darwin/config.nix;
  openssl = import ./holonix/openssl/config.nix;
  rust = import ./holonix/rust/config.nix;

in
with pkgs;
stdenv.mkDerivation rec {
  name = "holonix-shell";

  buildInputs = import ./holonix/build.nix;

  # non-nixos OS can have a "dirty" setup with rustup installed for the current
  # user.
  # `nix-shell` can inherit this e.g. through sourcing `.bashrc`.
  # even `nix-shell --pure` will still source some files and inherit paths.
  # for those users we can at least give the OS a clue that we want our pinned
  # rust version through this environment variable.
  # https://github.com/rust-lang/rustup.rs#environment-variables
  # https://github.com/NixOS/nix/issues/903
  RUSTUP_TOOLCHAIN = rust.nightly.version;
  RUSTFLAGS = rust.compile.flags;
  CARGO_INCREMENTAL = rust.compile.incremental;
  RUST_LOG = rust.log;
  NUM_JOBS = rust.compile.jobs;

  OPENSSL_STATIC = openssl.static;

  shellHook = ''
   # cargo installs things to the user's home so we need it on the path
   export PATH=~/.cargo/bin:$PATH
   export HC_TARGET_PREFIX=~/nix-holochain/
   export NIX_LDFLAGS="${darwin.ld-flags}$NIX_LDFLAGS"
  '';
}
