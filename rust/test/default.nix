{ holonix, pkgs }:
let
  hc-rust-test = pkgs.writeShellScriptBin "hc-rust-test"
  ''
  hc-rust-wasm-compile && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all "$1" -- --test-threads=${holonix.rust.test.threads};
  '';

  hc-rust-coverage-kcov = pkgs.writeShellScriptBin "hc-rust-coverage-kcov"
  ''
  nix-env -f https://github.com/NixOS/nixpkgs-channels/archive/nixos-19.09.tar.gz -iA kcov curl
  cargo install cargo-make
  hc-rust-wasm-compile
  for i in crates/*; do
    ( \
      cd $i && \
      export CARGO_TARGET_DIR=$(readlink -f ./target) && \
      cargo test --no-run && \
      cargo make codecov-flow \
    )
  done
  '';

  hc-rust-coverage-tarpaulin = pkgs.writeShellScriptBin "hc-rust-coverage-tarpaulin"
  ''
  unset CARGO_TARGET_DIR
  nix-env -f https://github.com/NixOS/nixpkgs-channels/archive/nixos-19.09.tar.gz -iA curl && \
    cargo install cargo-make || true && \
    cargo install cargo-tarpaulin || true && \
    hc-rust-wasm-compile && \
    cargo tarpaulin -o Xml
  '';
in
{
 buildInputs = [ hc-rust-test hc-rust-coverage-kcov hc-rust-coverage-tarpaulin ];
}
