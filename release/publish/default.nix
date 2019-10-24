{ pkgs, config }:
let
 name = "hc-release-hook-publish";

 script = pkgs.writeShellScriptBin name ''
set -euox pipefail
echo "packaging for crates.io"
# order is important here due to dependencies
for crate in \
 common \
 wasm_utils \
 conductor_api \
 dpki \
 sim2h \
 net \
 core \
 conductor_lib \
 core_types \
 cli \
 hdk \
 hdk_v2 \
 holochain \
 holochain_wasm \
 sim2h_server
do
 cargo publish --manifest-path "crates/$crate/Cargo.toml" --allow-dirty
 sleep 10
done
'';
in
{
 buildInputs = [ script ];
}
