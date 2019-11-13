{ pkgs }:
let
  name-trycp-server-install = "hc-trycp-server-install";
  script-trycp-server-install = pkgs.writeShellScriptBin name-trycp-server-install
  ''
  cd crates/trycp_server && cargo build -p trycp_server --release && cargo install --path . -f
  '';

  name-trycp-server = "hc-trycp-server";

  script-trycp-server = pkgs.writeShellScriptBin name-trycp-server ''
  set -euo pipefail
  RUST_LOG=debug trycp_server "$@"
  '';
in
{
  buildInputs = [ script-trycp-server-install script-trycp-server];
}
