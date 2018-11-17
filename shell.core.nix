# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

  date = "2018-10-12";
  wasmTarget = "wasm32-unknown-unknown";

  rust-build = (nixpkgs.rustChannelOfTargets "nightly" date [ wasmTarget ]);

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

  hc-test = nixpkgs.writeShellScriptBin "hc-test" "cargo test";
in
with nixpkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [
    rust-build

    hc-wasm-build
    hc-test

    zeromq
  ];

  shellHook =
  ''
  # https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
  export RUSTFLAGS="-D warnings"
  '';

}
