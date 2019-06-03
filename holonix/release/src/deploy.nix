let
  pkgs = import ../../nixpkgs/nixpkgs.nix;
  git = import ../../git/config.nix;
  release = import ../config.nix;

  name = "hc-release-deploy";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  echo "kicking off release"
  echo

  git checkout master
  git pull

  echo
  echo "releasing core ${release.core.tag}"
  echo

  echo "tagging ${release.core.tag}"
  git tag -a ${release.core.tag} -m "Version ${release.core.tag}"
  git push ${git.github.upstream} ${release.core.tag}

  echo
  echo "releasing node conductor ${release.node-conductor.tag}"
  echo

  echo "tagging ${release.node-conductor.tag}"
  git tag -a ${release.node-conductor.tag} -m "Node conductor version ${release.node-conductor.tag}"
  git push ${git.github.upstream} ${release.node-conductor.tag}

  echo "release tags pushed"
  echo "travis builds: https://travis-ci.com/holochain/holochain-rust/branches"
  echo "core artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.core.tag}"
  echo "nodejs artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.node-conductor.tag}"
  '';
in
script
