{ holonix, pkgs, config }:
let
 # build and push binaries to github from circle ci
 binaries-target = if pkgs.stdenv.isDarwin then holonix.rust.generic-mac-target else holonix.rust.generic-linux-target;

 binary-fns = import (builtins.fetchurl "https://gist.githubusercontent.com/thedavidmeister/077d825b367f1fb8a413936139856e20/raw/c78fed0aacf1e64900cc60ea5f63cb16e6ea8b92/default.nix");

 cli = {
  name = "cli";
  binary = "hc";
 };

 holochain = {
  name = "holochain";
  binary = "holochain";
 };

 sim2h-server = {
  name = "sim2h_server";
  binary = "sim2h_server";
 };

 trycp-server = {
  name = "trycp_server";
  binary = "trycp_server";
 };

 github-binary = args-raw:
 let
  args = args-raw // {
   version = config.release.tag;
   target = binaries-target;
  };
 in
 ''
 set -euox pipefail
 export artifact=${binary-fns.artifact-name args}
 echo ${args.name}
 cargo rustc --manifest-path crates/${args.name}/Cargo.toml --target ${args.target} --release -- -C lto
 mkdir -p $TMP/$artifact
 cp target/${args.target}/release/${args.binary} crates/${args.name}/LICENSE crates/${args.name}/README.md $TMP/$artifact/
 tar czf $TMP/$artifact.tar.gz -C $TMP/$artifact .
 github-release upload --file $TMP/$artifact.tar.gz --owner holochain --repo holochain-rust --tag ${args.version} --name $artifact.tar.gz --token $GITHUB_DEPLOY_TOKEN
 '';

 github-binaries = pkgs.writeShellScriptBin "hc-release-github-binaries"
 (pkgs.lib.concatStrings
  (map
   github-binary
   [
     cli
     holochain
     sim2h-server
     trycp-server
   ]));

 # order is important here due to dependencies
 publishable-crates = [
  "locksmith"
  "common"
  "metrics"
  "core_types"
  "wasm_utils"
  "conductor_api"
  "dpki"
  "sim2h"
  "sim1h"
  "net"
  "core"
  "conductor_lib"
  "hdk"
  "hdk_v2"
  "holochain"
  "holochain_wasm"
  "sim2h_server"
  ];

 remove-dev-deps = ''

 '';

 reinstate-dev-deps = ''
 git checkout -f
 '';

 publish-crates = dry:
 ''
 set -euox pipefail
 echo "packaging for crates.io"

 # remove dev deps
 cargo run --manifest-path crates/remove-dev-dependencies/Cargo.toml crates/**/Cargo.toml

 for crate in ${pkgs.lib.concatStrings (pkgs.lib.intersperse " " publishable-crates)}
 do
  echo "publishing ''${crate}"
  cargo publish --manifest-path "crates/$crate/Cargo.toml" --allow-dirty ${ if dry then "--dry-run --no-verify" else "" }
  sleep ${if dry then "1" else "10" }
 done

 # clean checkout
 git checkout -f
'';

 crates-io-dry-run = pkgs.writeShellScriptBin "hc-release-hook-publish-dry-run" (publish-crates true);
 crates-io = pkgs.writeShellScriptBin "hc-release-hook-publish" (publish-crates false);
in
{
 buildInputs = [ github-binaries crates-io-dry-run crates-io ];
}
