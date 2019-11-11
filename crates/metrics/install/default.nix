{ pkgs }:
let
  name-metrics-install = "hc-metrics-install";
  script-metrics-install = pkgs.writeShellScriptBin name-metrics-install
  ''
  cd crates/metrics && cargo build -p holochain_metrics --release && cargo install --path . -f
  '';

  name-metrics = "hc-metrics";

  script-metrics = pkgs.writeShellScriptBin name-metrics ''
  set -euo pipefail
  hc-metrics-install
  RUST_LOG=debug holochain_metrics "$@"
  '';
in
{
  buildInputs = [ script-metrics-install script-metrics];
}
