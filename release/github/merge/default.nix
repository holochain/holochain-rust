{ pkgs, github, release }:
let

  name = "hc-release-github-merge";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  echo 'ensure github PR'
  git config --local hub.upstream ${github.repo}
  git config --local hub.forkrepo ${github.repo}
  git config --local hub.forkremote ${github.upstream}
  if [ "$(git rev-parse --abbrev-ref HEAD)" == "${release.branch}" ]
   then
    git add . && git commit -am 'Release ${release.version.current}';
    git push;
    git pull ${github.upstream} master;
    git push ${github.upstream} master;
    git pull ${github.upstream} develop;
    git push ${github.upstream} develop;
   else
    echo "current branch is not ${release.branch}!";
    exit 1;
  fi
  '';
in
{
 buildInputs = [ script ];
}
