let
  pkgs = import ../../nixpkgs/nixpkgs.nix;
  coverage = import ./src/coverage.nix;
  install = import ./src/install.nix;
in
[
  # curl needed to push to codecov
  pkgs.curl

  coverage
  install
]
