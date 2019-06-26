{ pkgs, github, release }:
let

  name = "hc-release-github-merge";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  if [ "$(git rev-parse --abbrev-ref HEAD)" == "${release.branch}" ]
   then
    git add . && git commit -am 'Release ${release.version.current}';
    git push;
    git pull ${github.upstream} master;
    git push ${github.upstream} master;
    git checkout -b ${release.branch}-merge-back
    git pull ${github.upstream} develop;
    git push ${github.upstream} develop;
    git checkout ${release.branch}
   else
    echo "current branch is not ${release.branch}!";
    exit 1;
  fi
  '';
in
{
 buildInputs = [ script ];
}
