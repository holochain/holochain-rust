let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-scaffold-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-cli-install
  hc-conductor-node-install
  hc init /tmp/my_app
  ( cd /tmp/my_app && hc generate zomes/my_zome && hc test )
  '';
in
script
