let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-scaffold-test";

  script = pkgs.writeShellScriptBin name
  ''
  # build fresh hc and holochain
  hc-cli-install
  hc-conductor-node-install

  # init, test and cleanup a throwaway app
  app=`uuidgen`
  ( \
    cd /tmp \
    && hc init $app \
    && cd $app \
    && hc generate zomes/my_zome \
    && hc test \
    && rm -rf "/tmp/$app"
  )
  '';
in
script
