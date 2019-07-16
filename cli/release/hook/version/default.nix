{ pkgs, config }:
let
 name = "hc-cli-release-hook-version";

 script = pkgs.writeShellScriptBin name
 ''
 echo "bumping versions from ${config.release.version.previous} to ${config.release.version.current} in CLI"
 find . \
  -type f \
  -not -path "**/.git/**" \
  -path "./cli/*" | xargs -I {} \
  sed -i 's/${config.release.version.previous}/${config.release.version.current}/g' {}
 '';
in
{
 buildInputs = [ script ];
}
