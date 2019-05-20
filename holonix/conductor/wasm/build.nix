let
  compile = import ./src/compile.nix;
  install = import ./src/install.nix;
  test = import ./src/test.nix;
  uninstall = import ./src/uninstall.nix;
in
[
  compile
  install
  test
  uninstall
]
