let
  install = import ./src/install.nix;
  test = import ./src/test.nix;
in
[
  install
  test
]
