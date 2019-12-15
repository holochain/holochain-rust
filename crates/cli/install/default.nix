{ pkgs }:
let
  name = "hc-cli-install";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  CARGO_TARGET_DIR=$HC_TARGET_PREFIX/target/cli/install
  echo $CARGO_TARGET_DIR
  cargo install -f --path crates/cli
  '';
in
{
 buildInputs = [ script ];
}
