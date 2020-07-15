{ pkgs,config }:
let
 name = "hc-sim2h-server-perf";
  script-sim2h-server-perf = pkgs.writeShellScriptBin name
  ''
  cd crates/sim2h_server && cargo build -p sim2h_server --release && perf record --call-graph dwarf sim2h_server "$@"
  '';

  script-hc-conductor-perf = pkgs.writeShellScriptBin "hc-conductor-perf"
  ''
  cd crates/holochain && cargo build -p holochain --release && perf record --call-graph dwarf holochain
  '';

  script-hc-generate-flame-graph = pkgs.writeShellScriptBin "hc-generate-flame-graph"
  ''
  perf script | stackcollapse.pl | flamegraph.pl > generated-graph.svg
  '';
in
{
  buildInputs = [ script-sim2h-server-perf script-hc-generate-flame-graph script-hc-conductor-perf];
}
