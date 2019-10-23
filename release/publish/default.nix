{ pkgs, config }:
let
 name = "hc-release-hook-publish";

 script = pkgs.writeShellScriptBin name ''
set -euox pipefail
echo "packaging for crates.io"
# order is important here due to dependencies
for crate in \
 common \
 cli \
 conductor_api \
 conductor_lib \
 core \
 core_types \
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
