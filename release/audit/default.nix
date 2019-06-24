{ pkgs, release, pulse }:
let
 name = "hc-release-audit";

 script = pkgs.writeShellScriptBin name
 ''
 echo
 echo "Current git:"
 echo
 git show --pretty=oneline
 echo
 echo "All the important release vars:"
 echo
 echo "Target commit: ${release.commit}"
 echo
 echo "Dev pulse URL hash: ${pulse.url-hash}"
 echo "Dev pulse version: ${pulse.version}"
 echo "Dev pulse URL (derived): ${pulse.url}"
 echo
 echo "New core version: ${release.version.current}"
 echo "Previous core version: ${release.version.previous}"
 echo
 echo "Release process url: ${release.process-url}"
 '';
in
{
 buildInputs = [ script ];
}
