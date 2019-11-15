{pkgs }:
let
  name = "hc-rust-perf";

  script = pkgs.writeShellScriptBin name
  ''
  sudo apt-get install linux-tools-generic && cargo install flamegraph --force && sudo cargo flamegraph --test links
  '';
in
{
 buildInputs = [ script ];
}
