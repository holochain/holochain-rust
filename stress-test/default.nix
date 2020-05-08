{ pkgs }:
let
  script-stress-sim2h = pkgs.writeShellScriptBin "hc-stress-test-sim2h"
  ''
  hc-cli-install &&
  hc-conductor-install &&
  (
    cd stress-test &&
    wget https://github.com/holochain/passthrough-dna/releases/download/v0.0.8/passthrough-dna.dna.json -O passthrough-dna.dna.json
    npm install &&
    APP_SPEC_NETWORK_TYPE=sim2h TRYORAMA_CHOOSE_FREE_PORT=1 npm test -- "$@"
  )
  '';
in
{
 buildInputs = [ script-stress-sim2h pkgs.wget ];
}
