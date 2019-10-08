{ pkgs }:
let
  name-n3h = "hc-app-spec-test-n3h";

  script-n3h = pkgs.writeShellScriptBin name-n3h ''
  set -euo pipefail
  hc-cli-install
  hc-conductor-install
  (cd app_spec && APP_SPEC_NETWORK_TYPE="n3h" ./build_and_test.sh);
  '';

  name-memory = "hc-app-spec-test-memory";

  script-memory = pkgs.writeShellScriptBin name-memory ''
  set -euo pipefail
  hc-cli-install
  hc-conductor-install
  (cd app_spec && APP_SPEC_NETWORK_TYPE="memory" ./build_and_test.sh);
  '';

  name-sim1h = "hc-app-spec-test-sim1h";

  script-sim1h = pkgs.writeShellScriptBin name-sim1h ''
  set -euo pipefail
  hc-cli-install
  hc-conductor-install
  (cd app_spec && APP_SPEC_NETWORK_TYPE="sim1h" ./build_and_test.sh);
  '';

  name-sim2h = "hc-app-spec-test-sim2h";

  script-sim2h = pkgs.writeShellScriptBin name-sim2h ''
  set -euo pipefail
  hc-cli-install
  hc-conductor-install
  (cd app_spec && APP_SPEC_NETWORK_TYPE="sim2h" ./build_and_test.sh);
  '';

  name-sim2h-server = "hc-sim2h-server";

  script-sim2h-server = pkgs.writeShellScriptBin name-sim2h-server ''
  set -euo pipefail
  cargo install sim2h_server -f --git https://github.com/holochain/sim2h
  RUST_LOG=debug sim2h_server
  '';
in
{
 buildInputs = [ script-n3h script-memory script-sim1h script-sim2h script-sim2h-server ];
}
