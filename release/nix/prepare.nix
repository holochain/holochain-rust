{ pkgs }:
let
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

  hc-release-audit

  git hub --version

  echo
  read -r -p "Are you sure you want to cut a new release based on the current config in shell.nix? [y/N] " response
  case "$response" in
   [yY][eE][sS]|[yY])
    hc-release-pulse-tag \
    && hc-release-branch \
    && hc-release-rust-manifest-versions \
    && hc-release-docs-changelog-versions \
    && hc-release-github-merge \
    ;;
   *)
    exit 1
    ;;
  esac
  '';
in
script
