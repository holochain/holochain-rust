{ pkgs }:
let

  name = "hc-cli-test";

  script = pkgs.writeShellScriptBin name
  ''
  ( cd crates/cli && cargo test )
  bats crates/cli/test/hc.bats
  '';
in
{
 buildInputs = [ script ];
}
