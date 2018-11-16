# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

  coreDate = "2018-10-12";
  coreChannel = "nightly";
  coreToolchain = "${coreDate}-${coreChannel}";

  # toolsVersion = "nightly-2018-07-17";
  wasmTarget = "wasm32-unknown-unknown";

  rust-channel = (nixpkgs.rustChannelOf {
    date = coreDate;
    channel = coreChannel;
  });

  /* rust-wasm = (nixpkgs.rustChannelOf {
    date = coreDate;
    channel = coreChannel;
  }).rust; */
  rust-wasm = rust-channel.rust.override {
    targets = [ "wasm32-unknown-unknown" ];
  };

  # cargo = v: "rustup run ${v} cargo";
  # hc-fmt = pkgs.writeShellScriptBin "hc-fmt" "${cargo toolsVersion} fmt";
  hc-test = nixpkgs.writeShellScriptBin "hc-test" "cargo test";

  wasmBuild = path: "cargo build --release --target ${wasmTarget} --manifest-path ${path}";
  hc-wasm-build = nixpkgs.writeShellScriptBin "hc-wasm-build"
  ''
  ${wasmBuild "core/src/nucleus/wasm-test/Cargo.toml"}
  ${wasmBuild "core/src/nucleus/actions/wasm-test/Cargo.toml"}
  ${wasmBuild "container_api/wasm-test/round_trip/Cargo.toml"}
  ${wasmBuild "container_api/wasm-test/commit/Cargo.toml"}
  ${wasmBuild "hdk-rust/wasm-test/Cargo.toml"}
  ${wasmBuild "wasm_utils/wasm-test/integration-test/Cargo.toml"}
  '';
in
with nixpkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [
    # rust-nightly
    rust-wasm
    # pkgs.zeromq
    # pkgs.rustup
    # pkgs.cargo
    # pkgs.rustfmt
    # pkgs.rustc
    # pkgs.gnumake
    # pkgs.gcc
    # pkgs.binutils-unwrapped
    # pkgs.pkgconfig
    # pkgs.python27
    # rust

    # pkgs.libstdcxx5
    # pkgs.lld
    # pkgs.llvm

    # hc-fmt
    hc-wasm-build
    # hc-test
  ];

  /* shellHook =
  ''
  rustup toolchain install ${coreVersion}
  rustup toolchain install ${toolsVersion}
  rustup target add ${wasmTarget} --toolchain ${coreVersion}
  ''; */

}
