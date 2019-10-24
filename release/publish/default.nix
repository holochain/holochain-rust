{ pkgs, config }:
let
 name = "hc-release-hook-publish";

 script = pkgs.writeShellScriptBin name ''
set -euox pipefail
echo "packaging for crates.io"
# order is important here due to dependencies
# common \
for crate in \
 core_types \
 conductor_api \
 core \
 conductor_lib \
 cli \
 dpki \
 hdk \
 hdk_v2 \
 holochain \
 holochain_wasm \
 net \
 sim2h \
 sim2h_server \
 wasm_utils
do
 cargo publish --manifest-path "crates/$crate/Cargo.toml"
 sleep 10
done
'';
in
{
 buildInputs = [ script ];
}
