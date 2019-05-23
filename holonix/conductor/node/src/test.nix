let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-conductor-node-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-conductor-node-install && ( cd nodejs_conductor && npm test );
  '';
in
script
