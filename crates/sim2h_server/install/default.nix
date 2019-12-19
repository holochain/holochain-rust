{ pkgs }:
let
  script-sim2h-server-install = pkgs.writeShellScriptBin "hc-sim2h-server-install"
  ''
  cd crates/sim2h_server && cargo build -p sim2h_server --release && cargo install --path . -f
  '';

  script-sim2h-server = pkgs.writeShellScriptBin "hc-sim2h-server" ''
  set -euxo pipefail
  hc-sim2h-server-install
  RUST_LOG=debug sim2h_server "$@"
  '';
in
{
  buildInputs = [ script-sim2h-server-install script-sim2h-server];
}
