{ pkgs }:
let

  name = "hc-cli-test";

  script = pkgs.writeShellScriptBin name
  ''
  ( cd cli && cargo test )
  bats cli/test/hc.bats
  '';
in
{
 buildInputs = [ script ];
}
