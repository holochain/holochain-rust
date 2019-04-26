let
  dist = import ./src/dist.nix;
  install = import ./src/install.nix;
in
[
  dist
  install
]
