{ pkgs, release, pulse, github }:
let
  name = "hc-release-pulse-tag";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  echo 'tagging commit for pulse version ${pulse.version}'
  echo
  git fetch --tags
  if git tag | grep -q "${pulse.tag}"
   then
    echo "pulse tag for pulse ${pulse.version} already exists locally! doing nothing...";
    echo "pulse commit: $(git show-ref -s ${pulse.tag})";
    echo "to push upstream run: git push ${github.upstream} ${pulse.tag}";
   else
    echo "tagging...";
    git tag -a ${pulse.tag} ${release.commit} -m 'Dev pulse ${pulse.version}';
    echo "pushing...";
    git push ${github.upstream} ${pulse.tag};
    echo $'pulse tag ${pulse.tag} created and pushed';
  fi
  echo
  echo 'pulse tag on github: https://github.com/${github.repo}/${github.repo-name}/releases/tag/${pulse.tag}'
  echo
  '';
in
{
 buildInputs = [ script ];
}
