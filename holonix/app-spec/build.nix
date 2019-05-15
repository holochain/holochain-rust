let
  pkgs = import ../nixpkgs/nixpkgs.nix;

  serve = import ./src/serve.nix;
  test = import ./src/test.nix;
in
[
  serve
  test

  pkgs.expect
]
