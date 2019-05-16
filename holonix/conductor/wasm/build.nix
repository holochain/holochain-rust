let
  install = import ./src/install.nix;
  uninstall = import ./src/uninstall.nix;
  wasm = import ./src/wasm.nix;
in
[
  install
  uninstall
  wasm
]
