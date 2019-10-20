{ pkgs, config }:
let
 name = "sim2h-release-hook-version";

 script = pkgs.writeShellScriptBin name ''
for dep in \
 sim2h \
 sim2h_server
do
 echo "bumping $dep dependency versions to ${config.release.version.current} in all Cargo.toml"
 find . \
  -name "Cargo.toml" \
  -not -path "**/target/**" \
  -not -path "**/.git/**" \
  -not -path "**/.cargo/**" | xargs -I {} \
  sed -i 's/^'"''${dep}"' = { version = "=[0-9]\+.[0-9]\+.[0-9]\+\(-alpha[0-9]\+\)\?"/'"''${dep}"' = { version = "=${config.release.version.current}"/g' {}
done
'';
in
{
 buildInputs = [ script ];
}
