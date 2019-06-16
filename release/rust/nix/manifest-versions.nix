{ pkgs, release }:
let
 name = "hc-release-rust-manifest-versions";

 script = pkgs.writeShellScriptBin name
 ''
 echo "bumping core version from ${release.core.version.previous} to ${release.core.version.current} in Cargo.toml"
 find . \
  -name "Cargo.toml" \
  -not -path "**/.git/**" \
  -not -path "**/.cargo/**" \
  -not -path "./nodejs_conductor*" | xargs -I {} \
  sed -i 's/^\s*version\s*=\s*"${release.core.version.previous}"\s*$/version = "${release.core.version.current}"/g' {}
 echo "bumping core versions from ${release.core.version.previous} to ${release.core.version.current} in readmes"
 find . \
  -iname "readme.md" \
  -not -path "**/.git/**" \
  -not -path "**/.cargo/**" | xargs -I {} \
  sed -i 's/${release.core.version.previous}/${release.core.version.current}/g' {}
 echo "bumping versions from ${release.core.version.previous} to ${release.core.version.current} in CLI"
 find . \
  -type f \
  -not -path "**/.git/**" \
  -path "./cli/*" | xargs -I {} \
  sed -i 's/${release.core.version.previous}/${release.core.version.current}/g' {}
 echo "bumping node conductor version from ${release.node-conductor.version.previous} to ${release.node-conductor.version.current}"
 sed -i 's/^\s*version\s*=\s*"${release.node-conductor.version.previous}"\s*$/version = "${release.node-conductor.version.current}"/g' ./nodejs_conductor/native/Cargo.toml
 sed -i 's/"version": "${release.node-conductor.version.previous}"/"version": "${release.node-conductor.version.current}"/g' ./nodejs_conductor/package.json
 sed -i 's/"@holochain\/holochain-nodejs": "${release.node-conductor.version.previous}"/"@holochain\/holochain-nodejs": "${release.node-conductor.version.current}"/g' ./cli/src/cli/js-tests-scaffold/package.json
 '';
in
script
