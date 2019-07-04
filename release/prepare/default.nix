{ pkgs }:
let
  name = "hc-release-prepare";

  script = pkgs.writeShellScriptBin name
  ''
  hc-release-audit

  echo
  read -r -p "Are you sure you want to cut a new release based on the current config? [y/N] " response
  case "$response" in
   [yY][eE][sS]|[yY])
    hc-release-branch \
    && hc-release-rust-manifest-versions \
    && hc-release-docs-changelog-versions \
    && hc-release-github-merge
    ;;
   *)
    exit 1
    ;;
  esac
  '';
in
{
 buildInputs = [ script ];
}
