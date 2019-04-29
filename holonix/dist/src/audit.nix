let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  dist = import ../config.nix;

  name = "hc-dist-audit";

  script = pkgs.writeShellScriptBin name
  ''
   echo
   echo "All the important dist vars:"
   echo

   echo "Binary version is ${dist.version}"
  '';
in
script
