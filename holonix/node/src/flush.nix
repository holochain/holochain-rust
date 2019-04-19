let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-node-flush";

  script = pkgs.writeShellScriptBin name
  ''
   echo "flushing node artifacts"
   find . -wholename "**/node_modules" | xargs -I {} rm -rf  {};
   find . -wholename "./nodejs_conductor/bin-package" | xargs -I {} rm -rf {};
   find . -wholename "./nodejs_conductor/build" | xargs -I {} rm -rf {};
   find . -wholename "./nodejs_conductor/dist" | xargs -I {} rm -rf {};
  '';
in
script
