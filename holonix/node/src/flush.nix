let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-node-flush";

  script = pkgs.writeShellScriptBin name
  ''
   echo "flushing node artifacts"
   find . -wholename "**/node_modules" | xargs -I {} rm -rf  {};
  '';
in
script
