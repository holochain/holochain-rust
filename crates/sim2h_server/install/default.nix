{ pkgs }:
let
  name-sim2h-server-install = "hc-sim2h-server-install";
  script-sim2h-server-install = pkgs.writeShellScriptBin name-sim2h-server-install
  ''
  cd crates/sim2h_server && cargo build -p sim2h_server --release && cargo install --path . -f
  '';

  name-sim2h-server = "hc-sim2h-server";

  script-sim2h-server = pkgs.writeShellScriptBin name-sim2h-server ''
  set -euo pipefail
  hc-sim2h-server-install
  RUST_LOG=debug sim2h_server "$@"
  '';
in
{
  buildInputs = [ script-sim2h-server-install script-sim2h-server];
}
