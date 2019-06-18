{ pkgs, release, github }:
let

  name = "hc-release-branch";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  echo 'preparing release branch'
  echo
  git fetch
  if git tag | grep -q "${release.branch}"
  then echo "There is a tag with the same name as the release branch ${release.branch}! aborting..."
  exit 1;
  fi;
  echo
  echo 'checkout or create release branch'
  if git branch | grep -q "${release.branch}"
   then git checkout ${release.branch}
    git pull;
   else git checkout ${release.commit}
    git checkout -b ${release.branch}
    git push -u ${github.config.upstream} ${release.branch};
  fi;
  echo
  '';
in
script
