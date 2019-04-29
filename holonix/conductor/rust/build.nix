let
  dist = import ./src/dist.nix;
  install = import ./src/install.nix;
  uninstall = import ./src/uninstall.nix;
in
[
  dist
  install
  uninstall
]
