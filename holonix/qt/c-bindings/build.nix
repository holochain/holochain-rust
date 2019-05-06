let
  flush = import ./src/flush.nix;
  test = import ./src/test.nix;
in
[
  flush
  test
]
