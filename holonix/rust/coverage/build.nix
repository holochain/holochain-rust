let
  coverage = import ./src/coverage.nix;
  install = import ./src/install.nix;
in
[
  coverage
  install
]
