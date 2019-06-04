let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../../release/config.nix;
  dist = import ../../../dist/config.nix;

  name = "hc-conductor-node-dist";

  # the compiled node conductor needs to target specific node versions
  # at the moment this does nothing as i'm not sure how to override the node
  # dependency at this level in nix-shell yet
  node-versions = [ "nodejs-8_x" ];

  compile-node-conductor = node-version:
  ''
  hc-node-flush
  echo
  echo "building conductor for node ${node-version}..."
  echo

  node -v
  ./scripts/build_nodejs_conductor.sh
  cp nodejs_conductor/bin-package/index-v${release.node-conductor.version.current}-node-v57-linux-x64.tar.gz ${dist.path}
  '';

  script = pkgs.writeShellScriptBin name
  ''
  ${pkgs.lib.concatMapStrings (node-version: compile-node-conductor node-version) node-versions}
  '';
in
script
