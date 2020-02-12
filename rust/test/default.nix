{ holonix, pkgs }:
let
  hc-rust-test = pkgs.writeShellScriptBin "hc-rust-test"
  ''
  hc-rust-wasm-compile && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all "$1" -- --test-threads=${holonix.rust.test.threads};
  '';

  hc-rust-coverage-kcov = pkgs.writeShellScriptBin "hc-rust-coverage-kcov"
  ''
  # we need the kcov tool and curl for fetching the codecov.io uploader
  nix-env -f https://github.com/NixOS/nixpkgs-channels/archive/nixos-19.09.tar.gz -iA kcov curl

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
  bash <(curl -s https://codecov.io/bash)
  '';

  hc-rust-coverage-tarpaulin = pkgs.writeShellScriptBin "hc-rust-coverage-tarpaulin"
  ''
  unset CARGO_TARGET_DIR
  nix-env -f https://github.com/NixOS/nixpkgs-channels/archive/nixos-19.09.tar.gz -iA curl
  cargo install cargo-tarpaulin
  hc-rust-wasm-compile
  export CARGO_TARGET_DIR=$(readlink -f ./target)
  cargo tarpaulin -v -o Xml --exclude-files "*/.cargo/*"
  bash <(curl -s https://codecov.io/bash)
  '';
in
{
 buildInputs = [ hc-rust-test hc-rust-coverage-kcov hc-rust-coverage-tarpaulin ];
}
