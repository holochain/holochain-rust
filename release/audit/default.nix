{ pkgs, release }:
let
 name = "hc-release-audit";

 script = pkgs.writeShellScriptBin name
 ''
 echo
 echo "Current status of git"
 echo "This should be the latest develop commit, not necessarily the target commit below"
 echo
 git show --pretty=oneline
 echo
 echo "The important vars in ./release/config.nix:"
 echo
 echo "Target commit: ${release.commit}"
 echo "The target commit is the most recent commit on develop that passes test and starts with 'Merge pull request #XXX'"
 echo
 echo "New core version: ${release.version.current}"
 echo "Previous core version: ${release.version.previous}"
 echo
 echo "Release process url: ${release.process-url}"
 echo "You should be following the process documented at this URL right now"
 '';
in
{
 buildInputs = [ script ];
}
