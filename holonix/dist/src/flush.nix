let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  dist = import ../config.nix;

  name = "hc-dist-flush";

  script = pkgs.writeShellScriptBin name
  ''
   rm -rf ${dist.path}
  '';
in
script
