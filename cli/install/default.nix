{ pkgs }:
let
  name = "hc-cli-install";

  script = pkgs.writeShellScriptBin name
  ''
  cargo build -p hc --release && cargo install -f --path cli
  '';
in
{
 buildInputs = [ script ];
}
