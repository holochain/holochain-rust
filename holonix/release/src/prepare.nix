let
  pkgs = import ../../nixpkgs/nixpkgs.nix;
  release = import ../config.nix;

  name = "hc-release-prepare";

  script = pkgs.writeShellScriptBin name
  ''
   echo
   echo "IMPORTANT: make sure git-hub is setup on your machine"
   echo "1. Visit https://github.com/settings/tokens/new"
   echo "2. Generate a token called 'git-hub' with 'user' and 'repo' scopes"
   echo "3. git config --global hub.oauthtoken <token>"
   echo "4. git config --global hub.username <username>"
   echo
   echo "Current nix-shell config:"
   echo
   echo "pulse-url-hash: ${release.pulse.url-hash}"
   echo "pulse-version: ${release.pulse.version}"
   echo "pulse-commit: ${release.pulse.commit}"
   echo "core-previous-version: ${release.core.version.previous}"
   echo "core-version: ${release.core.version.current}"
   echo "node-conductor-previous-version: ${release.node-conductor.version.previous}"
   echo "node-conductor-version: ${release.node-conductor.version.current}"
   git hub --version
   echo
   read -r -p "Are you sure you want to cut a new release based on the current config in shell.nix? [y/N] " response
   case "$response" in
    [yY][eE][sS]|[yY])
     hc-release-pulse-tag \
     && hc-release-git-branch \
     && hc-prepare-crate-versions \
     && hc-ensure-changelog-version \
     && hc-prepare-release-pr \
     ;;
    *)
     exit 1
     ;;
   esac
  '';
in
script
