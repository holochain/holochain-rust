{ pkgs, config }:
let
 name = "hc-release-hook-publish";

 script = pkgs.writeShellScriptBin name ''
set -euox pipefail
echo "packaging for crates.io"

# regenerate this with `git diff >> release/publish/dev-dependencies.patch`
git apply release/publish/dev-dependencies.patch

# order is important here due to dependencies
for crate in \
 locksmith \
 common \
 core_types \
 wasm_utils \
 conductor_api \
 dpki \
 sim2h \
 sim1h \
 net \
 core \
 conductor_lib \
 hdk \
 hdk_v2 \
 holochain \
 holochain_wasm \
 sim2h_server
do
 cargo publish --manifest-path "crates/$crate/Cargo.toml" --allow-dirty
 sleep 10
done

git checkout -f
'';
in
{
 buildInputs = [ script ];
}
