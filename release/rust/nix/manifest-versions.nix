{ pkgs, release }:
let
 name = "hc-release-rust-manifest-versions";

 script = pkgs.writeShellScriptBin name
 ''
 echo "bumping core version from ${release.version.previous} to ${release.version.current} in Cargo.toml"
 find . \
  -name "Cargo.toml" \
  -not -path "**/.git/**" \
  -not -path "**/.cargo/**" | xargs -I {} \
  sed -i 's/^\s*version\s*=\s*"${release.version.previous}"\s*$/version = "${release.version.current}"/g' {}
 echo "bumping core versions from ${release.version.previous} to ${release.version.current} in readmes"
 find . \
  -iname "readme.md" \
  -not -path "**/.git/**" \
  -not -path "**/.cargo/**" | xargs -I {} \
  sed -i 's/${release.version.previous}/${release.version.current}/g' {}
 echo "bumping versions from ${release.version.previous} to ${release.version.current} in CLI"
 find . \
  -type f \
  -not -path "**/.git/**" \
  -path "./cli/*" | xargs -I {} \
  sed -i 's/${release.version.previous}/${release.version.current}/g' {}
 '';
in
script
