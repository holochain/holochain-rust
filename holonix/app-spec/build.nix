let
  serve = import ./src/serve.nix;
  test = import ./src/test.nix;
in
[
  serve
  test
]
