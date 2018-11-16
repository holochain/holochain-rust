# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

  rustChannel = "nightly";
  rustToolchain = date: "${rustChannel}-${date}";

  coreDate = "2018-10-12";
  coreToolchain = rustToolchain coreDate;

  rust-channel = (nixpkgs.rustChannelOf {
    date = coreDate;
    channel = rustChannel;
  });

  wasmTarget = "wasm32-unknown-unknown";
  rust-wasm = rust-channel.rust.override {
    targets = [ wasmTarget ];
  };

  toolsDate = "2018-07-17";
  toolsToolchain = rustToolchain toolsDate;
  rust-tools = (nixpkgs.rustChannelOf {
    date = toolsDate;
    channel = rustChannel;
  }).rust;

  cargo = date: "rustup run ${rustToolchain date} cargo";
  hc-fmt = nixpkgs.writeShellScriptBin "hc-fmt" "${cargo toolsDate} fmt";
  hc-test = nixpkgs.writeShellScriptBin "hc-test" "${cargo coreDate} test";

  wasmBuild = path: "${cargo coreDate} build --release --target ${wasmTarget} --manifest-path ${path}";
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
    rust-wasm
    rust-tools
    rustup

    hc-fmt
    hc-wasm-build
    hc-test
  ];

}
