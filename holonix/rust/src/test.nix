let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  rust = import ../config.nix;

  name = "hc-rust-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-rust-wasm-compile && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --release --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${rust.test.threads};
  '';
in
script
