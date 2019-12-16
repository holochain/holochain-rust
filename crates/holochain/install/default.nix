{ pkgs }:
let
  name = "hc-conductor-install";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  CARGO_TARGET_DIR=$CARGO_TARGET_DIR/holochain/install
  cargo install -f --path crates/holochain
  '';
in
{
 buildInputs = [ script ];
}
