{ holonix, pkgs, config }:
let
 # build and push binaries to github from circle ci
 binaries-target = if pkgs.stdenv.isDarwin then holonix.rust.generic-mac-target else holonix.rust.generic-linux-target;
 github-binaries = pkgs.writeShellScriptBin "hc-release-github-binaries" ''
 set -euox pipefail
 for p in cli holochain
 do
   nix-shell --run 'cargo rustc --manifest-path crates/$p/Cargo.toml --target ${binaries-target} --release -- -C lto'
   mkdir $p-$CIRCLE_TAG-${binaries-target}
   cp target/${binaries-target}/release/hc crates/$p/LICENSE crates/$p/README.md $p-$CIRCLE_TAG-${binaries-target}/
   tar czf $p-$CIRCLE_TAG-${binaries-target}.tar.gz $p-$CIRCLE_TAG-${binaries-target}/
   nix-shell --run "github-release upload --file ./$p-$CIRCLE_TAG-${binaries-target}.tar.gz --owner holochain --repo holochain-rust --tag $CIRCLE_TAG --name $p-$CIRCLE_TAG-${binaries-target}.tar.gz --token $GITHUB_DEPLOY_TOKEN"
 done
 '';

 crates-io = pkgs.writeShellScriptBin "hc-release-hook-publish" ''
set -euox pipefail
echo "packaging for crates.io"

cargo run --manifest-path crates/remove-dev-dependencies/Cargo.toml crates/**/Cargo.toml

# order is important here due to dependencies
for crate in \
 locksmith \
 common \
 metrics \
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
 buildInputs = [ github-binaries crates-io ];
}
