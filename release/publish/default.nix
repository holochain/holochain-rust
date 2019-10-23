{ pkgs, config }:
let
 name = "hc-release-hook-publish";

 script = pkgs.writeShellScriptBin name ''
set -euox pipefail
echo "packaging for crates.io"
# order is important here due to dependencies
for crate in \
 core_types \
 hdk \
 hdk_v2 \
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
