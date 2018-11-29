let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> {
    overlays = [ moz_overlay ];
  };

  date = "2018-10-12";
  wasmTarget = "wasm32-unknown-unknown";

  rust-build = (nixpkgs.rustChannelOfTargets "nightly" date [ wasmTarget ]);

  nodejs-8_13 = nixpkgs.nodejs-8_x.overrideAttrs(oldAttrs: rec {
    name = "nodejs-${version}";
    version = "8.13.0";
    src = nixpkgs.fetchurl {
      url = "https://nodejs.org/dist/v${version}/node-v${version}.tar.xz";
      sha256 = "1qidcj4smxsz3pmamg3czgk6hlbw71yw537h2jfk7iinlds99a9a";
    };
  });

  wasmBuild = path: "cargo build --release --target ${wasmTarget} --manifest-path ${path}";
  hc-wasm-build = nixpkgs.writeShellScriptBin "hc-wasm-build"
  ''
  ${wasmBuild "core/src/nucleus/actions/wasm-test/Cargo.toml"}
  ${wasmBuild "container_api/wasm-test/Cargo.toml"}
  ${wasmBuild "hdk-rust/wasm-test/Cargo.toml"}
  ${wasmBuild "wasm_utils/wasm-test/integration-test/Cargo.toml"}
  '';

  hc-test = nixpkgs.writeShellScriptBin "hc-test" "cargo test --all --exclude hc";

  hc-install-node-container = nixpkgs.writeShellScriptBin "hc-install-node-container" "cd nodejs_container && yarn install --ignore-scripts && node ./publish.js";
  hc-install-cmd = nixpkgs.writeShellScriptBin "hc-install-cmd" "cargo build -p hc && cargo install -f --path cmd";
  hc-test-cmd = nixpkgs.writeShellScriptBin "hc-test-cmd" "cd cmd && cargo test";

  hc-fmt = nixpkgs.writeShellScriptBin "hc-fmt" "cargo fmt";
  hc-fmt-check = nixpkgs.writeShellScriptBin "hc-fmt-check" "cargo fmt -- --check";
in
with nixpkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [
    # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md
    binutils gcc gnumake openssl pkgconfig
    carnix

    unixtools.watch

    cmake
    python
    pkgconfig
    zeromq
    rust-build

    nodejs-8_13
    yarn

    hc-wasm-build
    hc-test

    hc-install-node-container
    hc-install-cmd
    hc-test-cmd

    hc-fmt
    hc-fmt-check

    zeromq3
  ];

  # https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
  RUSTFLAGS = "-D warnings -Z external-macro-backtrace";

  shellHook = ''
  export PATH=$PATH:~/.cargo/bin;
  '';
}
