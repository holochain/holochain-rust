{ pkgs, release, github }:
let
  name = "hc-release-github-notes-sync";

  script = pkgs.writeShellScriptBin name
  ''
  export GITHUB_USER='${github.user}'
  export GITHUB_REPO='${github.repo-name}'
  export GITHUB_TOKEN=$( git config --get hub.oauthtoken )
  echo
  echo 'Injecting medium summary/highlights into github release notes'
  echo
  github-release -v edit --tag ${release.tag} --name ${release.tag} --description "$( hc-release-github-notes )" --pre-release
  '';
in
{
 buildInputs = [ script ];
}
