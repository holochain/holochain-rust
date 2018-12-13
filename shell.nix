let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> {
    overlays = [ moz_overlay ];
  };

  date = "2018-11-28";
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

  hc-flush-cargo-registry = nixpkgs.writeShellScriptBin "hc-flush-cargo-registry"
  ''
  rm -rf ~/.cargo/registry;
  rm -rf ~/.cargo/git;
  '';

  hc-test = nixpkgs.writeShellScriptBin "hc-test"
  ''
  cargo build --all --exclude hc;
  cargo test --all --exclude hc;
  '';

  hc-install-node-container = nixpkgs.writeShellScriptBin "hc-install-node-container"
  ''
  . ./scripts/build_nodejs_container.sh;
  '';

  hc-install-tarpaulin = nixpkgs.writeShellScriptBin "hc-install-tarpaulin" "if ! cargo --list | grep --quiet tarpaulin; then cargo install cargo-tarpaulin; fi;";
  hc-tarpaulin = nixpkgs.writeShellScriptBin "hc-tarpaulin" "cargo tarpaulin --ignore-tests --timeout 600 --all --out Xml --skip-clean -v -e holochain_core_api_c_binding -e hdk -e hc -e holochain_core_types_derive";

  hc-install-cmd = nixpkgs.writeShellScriptBin "hc-install-cmd" "cargo build -p hc && cargo install -f --path cmd";
  hc-test-cmd = nixpkgs.writeShellScriptBin "hc-test-cmd" "cd cmd && cargo test";
  hc-test-app-spec = nixpkgs.writeShellScriptBin "hc-test-app-spec" "cd app_spec && . build_and_test.sh";

  hc-fmt = nixpkgs.writeShellScriptBin "hc-fmt" "cargo fmt";
  hc-fmt-check = nixpkgs.writeShellScriptBin "hc-fmt-check" "cargo fmt -- --check";

  # runs all standard tests and reports code coverage
  ci-codecov = nixpkgs.writeShellScriptBin "ci-codecov"
  ''
  hc-wasm-build && \
  hc-install-tarpaulin && \
  hc-tarpaulin && \
  bash <(curl -s https://codecov.io/bash);
  '';

  # runs all app spec tests
  ci-app-spec = nixpkgs.writeShellScriptBin "ci-app-spec"
  ''
  hc-wasm-build && \
  hc-install-cmd && \
  hc-install-node-container && \
  hc-test-app-spec;
  '';

  # simulates all supported ci tests in a local circle ci environment
  ci = nixpkgs.writeShellScriptBin "ci" "circleci-cli local execute";

in
with nixpkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [
    # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md
    binutils gcc gnumake openssl pkgconfig coreutils
    # carnix

    cmake
    python
    pkgconfig
    zeromq
    rust-build

    nodejs-8_13
    yarn

    hc-flush-cargo-registry

    hc-wasm-build
    hc-test

    hc-install-tarpaulin
    hc-tarpaulin

    hc-install-cmd
    hc-install-node-container

    hc-test-cmd
    hc-test-app-spec

    hc-fmt
    hc-fmt-check

    zeromq3

    # dev tooling
    git
    docker

    # ci
    circleci-cli
    ci-codecov
    ci-app-spec
  ];


  # https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
  # https://llogiq.github.io/2017/06/01/perf-pitfalls.html
  # RUSTFLAGS = "-D warnings -Z external-macro-backtrace --cfg procmacro2_semver_exempt -C lto=no -Z incremental-info";
  RUSTFLAGS = "-D warnings -Z external-macro-backtrace --cfg procmacro2_semver_exempt";
  # CARGO_INCREMENTAL = "1";
  # https://github.com/rust-lang/cargo/issues/4961#issuecomment-359189913
  # RUST_LOG = "info";

  shellHook = ''
    # needed for install cmd and tarpaulin
    export PATH=$PATH:~/.cargo/bin;
  '';
}
