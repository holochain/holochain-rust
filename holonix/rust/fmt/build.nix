let
  check = import ./src/check.nix;
  fmt = import ./src/cmd.nix;
  install = import ./src/install.nix;
in
[
  check
  fmt
  install
]
