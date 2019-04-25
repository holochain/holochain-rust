let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-coverage-codecov";

  # runs all standard tests and reports code coverage
  script = pkgs.writeShellScriptBin name
  ''
   hc-rust-coverage-install \
   && hc-rust-coverage \
   && bash <(curl -s https://codecov.io/bash);
  '';
in
script
