let
  dist = import ./src/dist.nix;
  install = import ./src/install.nix;
  test = import ./src/test.nix;
  uninstall = import ./src/uninstall.nix;
in
[
  dist
  install
  test
  uninstall
]
