{ pkgs }:
let
  name = "hc-app-spec-test";

  script = pkgs.writeShellScriptBin name ''
  set -euo pipefail
  hc-cli-install
  hc-conductor-install
  (cd app_spec && ./build_and_test.sh);
  '';
in
{
 buildInputs = [ script ];
}
