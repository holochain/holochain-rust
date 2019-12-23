{ pkgs }:
let

 # (cd $app-spec-test-path && APP_SPEC_NETWORK_TYPE="$app-spec-test-network" ./build_and_test.sh);
 hc-test-app-spec = pkgs.writeShellScriptBin "hc-test-app-spec" ''
 set -euxo pipefail

 export app_spec_test_path="''${1:-app_spec}"
 export app_spec_test_network="''${2:-sim2h}"

 hc-cli-install
 hc-conductor-install

 (cd $app_spec_test_path && APP_SPEC_NETWORK_TYPE="$app_spec_test_network" ./build_and_test.sh)

 hc-cli-uninstall
 hc-conductor-uninstall
 '';
in
{
 buildInputs = [
  hc-test-app-spec
 ];
}
