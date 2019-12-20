{ pkgs }:
let
  name = "hc-cli-install";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  CARGO_TARGET_DIR=$CARGO_TARGET_DIR/cli/install
  cargo install -f --path crates/cli
  '';
in
{
 buildInputs = [ script ];
}
