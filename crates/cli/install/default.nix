{ pkgs }:
let
  name = "hc-cli-install";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  cargo install -f --path crates/cli
  '';
in
{
 buildInputs = [ script ];
}
