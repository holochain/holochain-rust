let
  check-version = import ./src/check-version.nix;
  deploy = import ./src/deploy.nix;
in
[
  check-version
  deploy
]
