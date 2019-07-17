{ pkgs }:
let

  name = "hc-cli-test";

  script = pkgs.writeShellScriptBin name
  ''
  (cd cli && cargo test);
  '';
in
{
 buildInputs = [ script ];
}
