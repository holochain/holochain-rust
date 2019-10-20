{ pkgs, config }:
let
 name = "sim2h-release-hook-publish";

 script = pkgs.writeShellScriptBin name ''
set -euox pipefail
echo "packaging for crates.io"
# order is important here due to dependencies
for crate in \
 sim2h \
 sim2h_server
do
 cargo publish --manifest-path "crates/$crate/Cargo.toml"

 sleep 10
done
'';
in
{
 buildInputs = [ script ];
}
