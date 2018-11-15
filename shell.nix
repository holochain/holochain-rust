# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
with import <nixpkgs> {};

let
  coreVersion = "nightly-2018-10-12";
  toolsVersion = "nightly-2018-07-17";
  wasmTarget = "wasm32-unknown-unknown";

  cargo = v: "rustup run ${v} cargo";
  hc-fmt = pkgs.writeShellScriptBin "hc-fmt" "${cargo toolsVersion} fmt";
  hc-test = pkgs.writeShellScriptBin "hc-test" "${cargo coreVersion} test";

  wasmBuild = path: "${cargo coreVersion} build --release --target ${wasmTarget} --manifest-path ${path}";
  hc-wasm-build = pkgs.writeShellScriptBin "hc-wasm-build"
  ''
  ${wasmBuild "core/src/nucleus/wasm-test/Cargo.toml"}
  ${wasmBuild "core/src/nucleus/actions/wasm-test/Cargo.toml"}
  ${wasmBuild "container_api/wasm-test/round_trip/Cargo.toml"}
  ${wasmBuild "container_api/wasm-test/commit/Cargo.toml"}
  ${wasmBuild "hdk-rust/wasm-test/Cargo.toml"}
  ${wasmBuild "wasm_utils/wasm-test/integration-test/Cargo.toml"}
  '';
in
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [
    pkgs.zeromq
    pkgs.rustup
    pkgs.cargo
    pkgs.rustfmt
    pkgs.rustc
    pkgs.gnumake
    pkgs.gcc
    pkgs.binutils-unwrapped
    pkgs.pkgconfig
    pkgs.python27
    
    pkgs.libstdcxx5
    pkgs.lld
    pkgs.llvm

    hc-fmt
    hc-wasm-build
    hc-test
  ];

  shellHook =
  ''
  rustup toolchain install ${coreVersion}
  rustup toolchain install ${toolsVersion}
  rustup target add ${wasmTarget} --toolchain ${coreVersion}
  '';

}
