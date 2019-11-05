{ holonix, pkgs, config }:
let
 # build and push binaries to github from circle ci
 binaries-target = if pkgs.stdenv.isDarwin then holonix.rust.generic-mac-target else holonix.rust.generic-linux-target;
 github-binaries = pkgs.writeShellScriptBin "hc-release-github-binaries" ''
 set -euox pipefail
 nix-shell --run 'cargo rustc --manifest-path crates/cli/Cargo.toml --target ${binaries-target} --release -- -C lto'
 mkdir cli-$CIRCLE_TAG-${binaries-target}
 cp target/${binaries-target}/release/hc crates/cli/LICENSE crates/cli/README.md cli-$CIRCLE_TAG-${binaries-target}/
 tar czf cli-$CIRCLE_TAG-${binaries-target}.tar.gz cli-$CIRCLE_TAG-${binaries-target}/
 nix-shell --run "github-release upload --file ./cli-$CIRCLE_TAG-${binaries-target}.tar.gz --owner holochain --repo holochain-rust --tag $CIRCLE_TAG --name cli-$CIRCLE_TAG-${binaries-target}.tar.gz --token $GITHUB_DEPLOY_TOKEN"
 '';

 crates-io = pkgs.writeShellScriptBin "hc-release-hook-publish" ''
set -euox pipefail
echo "packaging for crates.io"

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
'';
in
{
 buildInputs = [ github-binaries crates-io ];
}
