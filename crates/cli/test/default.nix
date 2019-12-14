{ pkgs }:
let

  name = "hc-cli-test";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  cargo test -p hc --target-dir "$HC_TARGET_PREFIX"/target/cli-test
  bats crates/cli/test/hc.bats
  '';
in
{
 buildInputs = [ script ];
}
