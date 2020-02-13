{ holonix, pkgs }:
let
  hc-rust-test = pkgs.writeShellScriptBin "hc-rust-test"
  ''
  hc-rust-wasm-compile && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all "$1" -- --test-threads=${holonix.rust.test.threads};
  '';

  hc-rust-coverage-kcov = pkgs.writeShellScriptBin "hc-rust-coverage-kcov"
  ''
  # this script is a little messy - mainly only designed to be run on CI
  # cargo-make coverage only really works for individual crates so we futz
  # with the CARGO_TARGET_DIR - also we just blanket install cargo-make

  # using cargo-make for executing kcov
  cargo install cargo-make

  # some tests require compiled wasm
  hc-rust-wasm-compile

  # kcov does not work with the global /holochain-rust/target
  mkdir -p target

  # actually kcov does not work with workspace target either
  # we need to use targets in each crate - but that is slow
  # use symlinks so we don't have to recompile deps over and over
  for i in ''$(find crates -maxdepth 1 -mindepth 1 -type d | sort); do

    # delete all other test binaries so they don't get run multiple times
    rm -rf $(find target/debug -maxdepth 1 -mindepth 1 -type f)

    echo "-------"
    echo "coverage for '$i'"
    echo "-------"

    # cd into crate dirs
    # remove any pre-existing target dir
    # symlink to shared target dir
    # build the test binaries
    # run the code coverage

    ( \
      cd $i && \
      rm -rf target || true && \
      ln -s ../../target
      export CARGO_TARGET_DIR=$(readlink -f ./target) && \
      cargo test --no-run && \
      cargo make coverage-kcov \
    )

    # then remove the symlink so we don't double count the coverage reports
    rm -rf "''${i}/target"
  done

  # upload to codecove.io
  bash <(curl -s https://codecov.io/bash) -t "''${CODECOV_TOKEN}"
  '';
in
{
 buildInputs = [ pkgs.kcov pkgs.curl hc-rust-test hc-rust-coverage-kcov ];
}
