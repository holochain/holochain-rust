let

  pkgs = import ./holonix/nixpkgs/nixpkgs.nix;
  rust = import ./holonix/rust/config.nix;
  release = import ./holonix/release/config.nix;
  git = import ./holonix/git/config.nix;

in
with pkgs;
stdenv.mkDerivation rec {
  name = "holonix-shell";

  buildInputs = import ./holonix/build.nix;

  RUSTFLAGS = rust.compile.flags;
  CARGO_INCREMENTAL = rust.compile.incremental;
  RUST_LOG = rust.log;
  NUM_JOBS = rust.compile.jobs;

  # non-nixos OS can have a "dirty" setup with rustup installed for the current
  # user.
  # `nix-shell` can inherit this e.g. through sourcing `.bashrc`.
  # even `nix-shell --pure` will still source some files and inherit paths.
  # for those users we can at least give the OS a clue that we want our pinned
  # rust version through this environment variable.
  # https://github.com/rust-lang/rustup.rs#environment-variables
  # https://github.com/NixOS/nix/issues/903
  RUSTUP_TOOLCHAIN = "${rust.nightly.version}";

  DARWIN_NIX_LDFLAGS = if stdenv.isDarwin then "-F${frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation " else "";

  OPENSSL_STATIC = "1";

  shellHook = ''
   # cargo installs things to the user's home so we need it on the path
   export PATH=$PATH:~/.cargo/bin
   export HC_TARGET_PREFIX=~/nix-holochain/
   export NIX_LDFLAGS="$DARWIN_NIX_LDFLAGS$NIX_LDFLAGS"
  '';
}
