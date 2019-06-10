let
 holonix-release-tag = "expecting-bad-ozone";
 holonix-release-sha256 = "0x6bma8p1kmlzsr2dd5zjyvw809hjmycdk8n8bknigx7vd88v52m";

 holonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/archive/${holonix-release-tag}.tar.gz";
  sha256 = "${holonix-release-sha256}";
 });
in
with holonix.pkgs;
{
 core-shell =
  stdenv.mkDerivation rec {
    name = "core-shell";

    buildInputs = holonix.buildInputs;

    # non-nixos OS can have a "dirty" setup with rustup installed for the current
    # user.
    # `nix-shell` can inherit this e.g. through sourcing `.bashrc`.
    # even `nix-shell --pure` will still source some files and inherit paths.
    # for those users we can at least give the OS a clue that we want our pinned
    # rust version through this environment variable.
    # https://github.com/rust-lang/rustup.rs#environment-variables
    # https://github.com/NixOS/nix/issues/903
    RUSTUP_TOOLCHAIN = holonix.rust.nightly.version;
    RUSTFLAGS = holonix.rust.compile.flags;
    CARGO_INCREMENTAL = holonix.rust.compile.incremental;
    RUST_LOG = holonix.rust.log;
    NUM_JOBS = holonix.rust.compile.jobs;

    OPENSSL_STATIC = holonix.openssl.static;

    shellHook = ''
     # cargo should install binaries into this repo rather than globally
     # https://github.com/rust-lang/rustup.rs/issues/994
     export CARGO_HOME=`pwd`/.cargo
     export CARGO_INSTALL_ROOT=`pwd`/.cargo
     export PATH="$CARGO_INSTALL_ROOT/bin:$PATH"

     export HC_TARGET_PREFIX=~/nix-holochain/
     export NIX_LDFLAGS="${holonix.darwin.ld-flags}$NIX_LDFLAGS"
    '';
  };

  hc = holonix.hc;
  holochain = holonix.holochain;
}
