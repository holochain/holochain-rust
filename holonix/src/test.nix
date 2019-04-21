let
  pkgs = import ../nixpkgs/nixpkgs.nix;

  name = "hc-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-rust-fmt-check \
  && hc-qt-c-bindings-test \
  && hc-rust-wasm-compile \
  && hc-app-spec-test
  '';
in
script
