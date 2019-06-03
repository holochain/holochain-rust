# simplified version of the c bindings test command in makefile
# hardcodes hc_dna to test rather than looping/scanning like make does
# might want to make this more sophisticated if we end up with many tests
let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-qt-c-bindings-flush";

  script = pkgs.writeShellScriptBin name
  ''
  rm c_binding_tests/hc_dna/.qmake.stash
  rm c_binding_tests/hc_dna/Makefile
  '';
in
script
