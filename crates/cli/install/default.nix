{ pkgs }:
let
  name = "hc-cli-install";

  script = pkgs.writeShellScriptBin name
  ''
  cd crates/cli && cargo build -p hc --release && cargo install -f
  '';
in
{
 buildInputs = [ script ];
}
