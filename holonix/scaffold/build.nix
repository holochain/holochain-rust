let
  pkgs = import ../nixpkgs/nixpkgs.nix;

  test = import ./src/test.nix;
in
[
  # provides uuidgen used in scaffold testing
  pkgs.utillinux

  test
]
