let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-scaffold-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-cli-install
  hc-conductor-node-install

  ( \
    cd /tmp \
    && hc init my_app \
    && cd my_app \
    && hc generate zomes/my_zome \
    && hc test \
  )
  '';
in
script
