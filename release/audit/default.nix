{ pkgs, config }:
let
 name = "hc-release-audit";

 script = pkgs.writeShellScriptBin name
 ''
 echo
 echo "Current status of git"
 echo "This should be the latest develop commit, not necessarily the target commit below"
 echo
 echo "The important vars in ./config.nix:"
 echo
 echo "~HEAD~ commit: $( git rev-parse --verify HEAD )"
 echo "CONFIG commit: ${config.release.commit}"
 echo "The target commit is the most recent commit on develop that passes test and starts with 'Merge pull request #XXX'"
 echo
 echo "New core version: ${config.release.version.current}"
 echo "Previous core version: ${config.release.version.previous}"
 '';
in
{
 buildInputs = [ script ];
}
