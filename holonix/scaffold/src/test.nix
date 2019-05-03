let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-scaffold-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-cli-install
  hc-conductor-node-install
  hc init my_app
  cd my_app
  echo $USER
  export USER=$USER
  hc generate zomes/my_zome
  hc test
  '';
in
script
