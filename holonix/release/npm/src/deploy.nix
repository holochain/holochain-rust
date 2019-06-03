let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../config.nix;

  name = "hc-release-npm-deploy";

  script = pkgs.writeShellScriptBin name
  ''
  git checkout holochain-nodejs-v${release.node-conductor.version.current}
  npm login
  cd nodejs_conductor
  yarn install --ignore-scripts
  RUST_SODIUM_DISABLE_PIE=1 node ./publish.js --publish
  '';
in
script
