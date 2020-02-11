{ holonix, pkgs }:
let
  hc-rust-test = pkgs.writeShellScriptBin "hc-rust-test"
  ''
  hc-rust-wasm-compile && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all "$1" -- --test-threads=${holonix.rust.test.threads};
  '';

  hc-rust-coverage = pkgs.writeShellScriptBin "hc-rust-coverage"
  ''
  nix-env -f https://github.com/NixOS/nixpkgs-channels/archive/nixos-19.09.tar.gz -iA kcov && \
    cargo install cargo-make || true && \
    cargo test --no-run && \
    CARGO_MAKE_WORKSPACE_TARGET_DIRECTORY=$(readlink -f ./target) cargo make codecov-flow
  '';
in
{
 buildInputs = [ hc-rust-test hc-rust-coverage ];
}
