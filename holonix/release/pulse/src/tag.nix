let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../config.nix;
  release-pulse = import ../config.nix;
  git = import ../../../git/config.nix;

  name = "hc-release-pulse-tag";

  script = pkgs.writeShellScriptBin name
  ''
   echo
   echo 'tagging commit for pulse version ${release-pulse.version}'
   echo
   git fetch --tags
   if git tag | grep -q "${release-pulse.tag}"
    then
     echo "pulse tag for pulse ${release-pulse.version} already exists locally! doing nothing...";
     echo "pulse commit: $(git show-ref -s ${release-pulse.tag})";
     echo "to push upstream run: git push ${git.github.upstream} ${release-pulse.tag}";
    else
     echo "tagging...";
     git tag -a ${release-pulse.tag} ${release.commit} -m 'Dev pulse ${release-pulse.version}';
     echo "pushing...";
     git push ${git.github.upstream} ${release-pulse.tag};
     echo $'pulse tag ${release-pulse.tag} created and pushed';
   fi
   echo
   echo 'pulse tag on github: https://github.com/holochain/holochain-rust/releases/tag/${release-pulse.tag}'
   echo
  '';
in
script
