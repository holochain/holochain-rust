let
 pkgs = 

[
upstream = "origin";
pulse-version = "22";
pulse-tag = "dev-pulse-${pulse-version}";
pulse-commit = "0a524d3be580249d54cf5073591fa9fe1f30a174";
core-version = "0.0.9-alpha";
node-conductor-version = "0.4.8-alpha";
hc-prepare-pulse-tag = pkgs.writeShellScriptBin
hc-prepare-release = pkgs.writeShellScriptBin "hc-prepare-release"
''
 echo $'\ntagging commit for pulse version ${pulse-version}\n'
 git fetch --tags
 if git tag | grep -q "${pulse-tag}"
  then
   echo "pulse tag for pulse ${pulse-version} already exists locally! doing nothing..."
   echo "pulse commit: $(git show-ref -s ${pulse-tag})"
   echo "to push upstream run: git push ${upstream} ${pulse-tag}"
  else
   echo "tagging..."
   git tag -a ${pulse-tag} ${pulse-commit} -m 'Dev pulse ${pulse-version}'
   echo "pushing..."
   git push ${upstream} ${pulse-tag}
   echo $'pulse tag ${pulse-tag} created and pushed'
 fi
 echo $'\npulse tag on github: https://github.com/holochain/holochain-rust/releases/tag/${pulse-tag}\n'

 echo $'\nensuring release branch\n'
'';
]
