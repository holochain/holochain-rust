let
  dist = import ./src/dist.nix;
  test = import ./src/test.nix;
in
[
  dist
  test
]
