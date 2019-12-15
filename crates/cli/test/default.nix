{ pkgs }:
let

  name = "hc-cli-test";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  cargo test -p hc --target-dir "$CARGO_TARGET_DIR"/cli-test
  bats crates/cli/test/hc.bats
  '';
in
{
 buildInputs = [ script ];
}
