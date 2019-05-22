let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../config.nix;
  git = import ../../../git/config.nix;

  name = "hc-release-github-merge-back";

  script = pkgs.writeShellScriptBin name
  ''
   echo
   echo 'ensure github PR against develop'
   echo
   git config --local hub.upstream ${git.github.repo}
   git config --local hub.forkrepo ${git.github.repo}
   git config --local hub.forkremote ${git.github.upstream}
   if [ "$(git rev-parse --abbrev-ref HEAD)" == "${release.branch}" ]
    then
     git add . && git commit -am 'Release ${release.core.version.current}';
     git push && git hub pull new -b 'develop' -m 'Merge release ${release.core.version.current} back to develop' --no-triangular ${release.branch};
    else
     echo "current branch is not ${release.branch}!";
     exit 1;
   fi
  '';
in
script
