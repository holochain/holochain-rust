{ pkgs, release, github }:
let
  name = "hc-release-deploy";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  echo "kicking off release"
  echo
  git checkout master
  git pull
  echo
  echo "releasing core ${release.tag}"
  echo
  echo "tagging ${release.tag}"
  git tag -a ${release.tag} -m "Version ${release.tag}"
  git push ${github.config.upstream} ${release.tag}
  echo
  echo "release tags pushed"
  echo "travis builds: https://travis-ci.com/holochain/holochain-rust/branches"
  echo "core artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.tag}"
  '';
in
script
