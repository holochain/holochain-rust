{ pkgs, config }:
let
 name = "hc-release-hook-publish";

 script = pkgs.writeShellScriptBin name ''
set -euox pipefail
echo "packaging for crates.io"
# order is important here due to dependencies
for crate in \
 cli \
 conductor_api \
 conductor_lib \
 dpki \
 hdk \
 hdk_v2 \
 holochain \
 holochain_wasm \
 net \
 wasm_utils \
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
