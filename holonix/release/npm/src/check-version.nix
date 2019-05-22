let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../config.nix;

  name = "hc-release-npm-check-version";

  script = pkgs.writeShellScriptBin name
  ''
   echo
   echo "Checking deployed nodejs_conductor version."
   deployed=$( npm v @holochain/holochain-nodejs dist-tags.latest )
   if [ $deployed == ${release.node-conductor.version.current} ]
    then echo "Version ${release.node-conductor.version.current} deployed ✔";
    else echo "Not deployed. $deployed found instead. ⨯";
   fi
   echo
  '';
in
script
