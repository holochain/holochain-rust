let
  pkgs = import ../../nixpkgs/nixpkgs.nix;
  codecov = import ./src/codecov.nix;
  coverage = import ./src/coverage.nix;
  install = import ./src/install.nix;
in
[
  # curl needed to push to codecov
  pkgs.curl

  codecov
  coverage
  install
]
