let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> {
    overlays = [ moz_overlay ];
  };

  date = "2018-12-26";
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

  hc-flush-cargo-registry = nixpkgs.writeShellScriptBin "hc-flush-cargo-registry"
  ''
  rm -rf ~/.cargo/registry;
  rm -rf ~/.cargo/git;
  '';

  hc-install-node-container = nixpkgs.writeShellScriptBin "hc-install-node-container"
  ''
  . ./scripts/build_nodejs_container.sh;
  '';

  hc-install-tarpaulin = nixpkgs.writeShellScriptBin "hc-install-tarpaulin"
  ''
  if ! cargo --list | grep --quiet tarpaulin;
  then
    RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin;
  fi;
  '';
  hc-tarpaulin = nixpkgs.writeShellScriptBin "hc-tarpaulin" "cargo tarpaulin --ignore-tests --timeout 600 --all --out Xml --skip-clean -v -e holochain_core_api_c_binding -e hdk -e hc -e holochain_core_types_derive";

  hc-install-cmd = nixpkgs.writeShellScriptBin "hc-install-cmd" "cargo build -p hc --release && cargo install -f --path cmd";
  hc-test-cmd = nixpkgs.writeShellScriptBin "hc-test-cmd" "cd cmd && cargo test";
  hc-test-app-spec = nixpkgs.writeShellScriptBin "hc-test-app-spec" "cd app_spec && . build_and_test.sh";

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
  cargo build --release --target wasm32-unknown-unknown --manifest-path "$TEST_PATH""$WASM_PATH"Cargo.toml --target-dir "$HC_TARGET_PREFIX""$TEST_PATH""$WASM_PATH"target;
  '';
  test = test-p: test-path: wasm-paths:
  ''
   export TEST_PATH=${test-path}/;
   ${nixpkgs.lib.concatMapStrings (path: build-wasm path) wasm-paths}
   cargo test -p ${test-p} --release --target-dir "$HC_TARGET_PREFIX""$TEST_PATH"target -- --nocapture;
  '';
  hc-test-hdk = nixpkgs.writeShellScriptBin "hc-test-hdk" "${test "hdk" "hdk-rust" [ "wasm-test" ]}";
  hc-test-wasm-utils = nixpkgs.writeShellScriptBin "hc-test-wasm-utils" "${test "holochain_wasm_utils" "wasm_utils" [ "wasm-test/integration-test" ]}";
  hc-test-container-api = nixpkgs.writeShellScriptBin "hc-test-container-api" "${test "holochain_container_api" "container_api" [ "wasm-test" "test-bridge-caller" ]}";
  hc-test-core = nixpkgs.writeShellScriptBin "hc-test-core" "${test "holochain_core" "core" [ "src/nucleus/actions/wasm-test" ]}";
  hc-test-cas-implementations = nixpkgs.writeShellScriptBin "hc-test-cas-implementations" "${test "holochain_cas_implementations" "cas_implementations" [] }";
  hc-test-dna-c-binding = nixpkgs.writeShellScriptBin "hc-test-dna-c-binding" "${test "holochain_dna_c_binding" "dna_c_binding" []}";
  hc-test-net-connection = nixpkgs.writeShellScriptBin "hc-test-net-connection" "${test "holochain_net_connection" "net_connection" []}";
  hc-test-sodium = nixpkgs.writeShellScriptBin "hc-test-sodium" "${test "holochain_sodium" "sodium" []}";
  hc-test-hc = nixpkgs.writeShellScriptBin "hc-test-hc" "${test "hc" "cmd" []}";
  hc-test-core-types = nixpkgs.writeShellScriptBin "hc-test-core-types" "${test "holochain_core_types" "core_types" []}";
  hc-test-net = nixpkgs.writeShellScriptBin "hc-test-net" "${test "holochain_net" "net" []}";
  hc-test-net-ipc = nixpkgs.writeShellScriptBin "hc-test-net-ipc" "${test "holochain_net_ipc" "net_ipc" []}";
  hc-test = nixpkgs.writeShellScriptBin "hc-test"
  ''
  hc-test-hdk \
  && hc-test-wasm-utils \
  && hc-test-container-api \
  && hc-test-core \
  && hc-test-cas-implementations \
  && hc-test-dna-c-binding \
  && hc-test-net-connection \
  && hc-test-sodium \
  && hc-test-hc \
  && hc-test-core-types \
  && hc-test-net \
  && hc-test-net-ipc \
  ;
  '';

  flush = nixpkgs.writeShellScriptBin "flush"
  ''
  rm -rf target
  rm -rf .cargo
  rm -rf **/target
  rm -rf **/.cargo
  rm -rf **/**/target
  rm -rf **/**/.cargo
  rm -rf **/**/**/target
  rm -rf **/**/**/.cargo
  rm -rf **/**/**/**/target
  rm -rf **/**/**/**/.cargo
  '';

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
    rust-build

    nodejs-8_13
    yarn

    hc-flush-cargo-registry

    hc-test

    hc-install-tarpaulin
    hc-tarpaulin

    hc-install-cmd
    hc-install-node-container

    hc-test-cmd
    hc-test-app-spec

    hc-fmt
    hc-fmt-check

    zeromq4

    # dev tooling
    git

    # curl needed to push to codecov
    curl
    docker
    circleci-cli
    hc-codecov
    ci

    hc-test-hdk
    hc-test-wasm-utils
    hc-test-container-api
    hc-test-core
    hc-test-cas-implementations
    hc-test-dna-c-binding
    hc-test-net-connection
    hc-test-sodium
    hc-test-hc
    hc-test-core-types
    hc-test-net
    hc-test-net-ipc
    flush
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
    # needed for install cmd and tarpaulin
    export PATH=$PATH:~/.cargo/bin;
    export HC_TARGET_PREFIX=/tmp/holochain/
  '';
}
