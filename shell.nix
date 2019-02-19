let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> {
    overlays = [ moz_overlay ];
  };

  date = "2019-01-24";
  wasmTarget = "wasm32-unknown-unknown";

  rust-build = (nixpkgs.rustChannelOfTargets "nightly" date [ wasmTarget ]);

  hc-flush-cargo-registry = nixpkgs.writeShellScriptBin "hc-flush-cargo-registry"
  ''
   rm -rf ~/.cargo/registry;
   rm -rf ~/.cargo/git;
  '';

  hc-install-node-conductor = nixpkgs.writeShellScriptBin "hc-install-node-conductor"
  ''
   export RUST_SODIUM_LIB_DIR=/nix/store/l1nbc3vgr37lswxny8pwhkq4m937y2g4-libsodium-1.0.16;
   export RUST_SODIUM_SHARED=1;
   . ./scripts/build_nodejs_conductor.sh;
  '';

  hc-install-tarpaulin = nixpkgs.writeShellScriptBin "hc-install-tarpaulin"
  ''
   if ! cargo --list | grep --quiet tarpaulin;
   then
    RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin;
   fi;
  '';
  hc-tarpaulin = nixpkgs.writeShellScriptBin "hc-tarpaulin" "cargo tarpaulin --ignore-tests --timeout 600 --all --out Xml --skip-clean -v -e holochain_core_api_c_binding -e hdk -e hc -e holochain_core_types_derive";

  hc-install-cli = nixpkgs.writeShellScriptBin "hc-install-cli" "cargo build -p hc --release && cargo install -f --path cli";
  hc-install-conductor = nixpkgs.writeShellScriptBin "hc-install-conductor" "cargo build -p holochain --release && cargo install -f --path conductor";

  hc-test-cli = nixpkgs.writeShellScriptBin "hc-test-cli" "cd cli && cargo test";
  hc-test-app-spec = nixpkgs.writeShellScriptBin "hc-test-app-spec" "cd app_spec && . build_and_test.sh";
  hc-test-node-conductor = nixpkgs.writeShellScriptBin "hc-test-node-conductor" "cd nodejs_conductor && npm test";

  hc-fmt = nixpkgs.writeShellScriptBin "hc-fmt" "cargo fmt";
  hc-fmt-check = nixpkgs.writeShellScriptBin "hc-fmt-check" "cargo fmt -- --check";

  # runs all standard tests and reports code coverage
  hc-codecov = nixpkgs.writeShellScriptBin "hc-codecov"
  ''
   hc-install-tarpaulin && \
   hc-tarpaulin && \
   bash <(curl -s https://codecov.io/bash);
  '';

  # simulates all supported ci tests in a local circle ci environment
  ci = nixpkgs.writeShellScriptBin "ci"
  ''
   circleci-cli local execute
  '';

  build-wasm = wasm-path:
  ''
   export WASM_PATH=${wasm-path}/
   cargo build --release --target wasm32-unknown-unknown --manifest-path "$WASM_PATH"Cargo.toml --target-dir "$HC_TARGET_PREFIX""$WASM_PATH"target;
  '';
  wasm-paths = [
   "hdk-rust/wasm-test"
   "wasm_utils/wasm-test/integration-test"
   "conductor_api/wasm-test"
   "conductor_api/test-bridge-caller"
   "core/src/nucleus/actions/wasm-test"
  ];
  hc-build-wasm = nixpkgs.writeShellScriptBin "hc-build-wasm"
  ''
   ${nixpkgs.lib.concatMapStrings (path: build-wasm path) wasm-paths}
  '';
  hc-test = nixpkgs.writeShellScriptBin "hc-test"
  ''
   hc-build-wasm
   HC_SIMPLE_LOGGER_MUTE=1 cargo test --all --release --target-dir "$HC_TARGET_PREFIX"target;
  '';

in
with nixpkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [

    # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md
    binutils gcc gnumake openssl pkgconfig coreutils

    cmake
    python
    pkgconfig
    rust-build

    nodejs-8_x
    yarn

    hc-flush-cargo-registry

    hc-build-wasm
    hc-test

    hc-install-tarpaulin
    hc-tarpaulin

    hc-install-cli
    hc-install-conductor
    hc-install-node-conductor

    hc-test-cli
    hc-test-app-spec
    hc-test-node-conductor

    hc-fmt
    hc-fmt-check

    zeromq4
    libsodium

    # dev tooling
    git

    # curl needed to push to codecov
    curl
    circleci-cli
    hc-codecov
    ci
  ];

  # https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
  # https://llogiq.github.io/2017/06/01/perf-pitfalls.html
  RUSTFLAGS = "-D warnings -Z external-macro-backtrace -Z thinlto -C codegen-units=16 -C opt-level=z";
  CARGO_INCREMENTAL = "1";
  # https://github.com/rust-lang/cargo/issues/4961#issuecomment-359189913
  # RUST_LOG = "info";

  # non-nixos OS can have a "dirty" setup with rustup installed for the current
  # user.
  # `nix-shell` can inherit this e.g. through sourcing `.bashrc`.
  # even `nix-shell --pure` will still source some files and inherit paths.
  # for those users we can at least give the OS a clue that we want our pinned
  # rust version through this environment variable.
  # https://github.com/rust-lang/rustup.rs#environment-variables
  # https://github.com/NixOS/nix/issues/903
  RUSTUP_TOOLCHAIN = "nightly-${date}";

  shellHook = ''
   # needed for install cli and tarpaulin
   export PATH=$PATH:~/.cargo/bin;
   export HC_TARGET_PREFIX=~/nix-holochain/
  '';
}
