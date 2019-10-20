{ pkgs }:
let
  name = "hc-test";

  script = pkgs.writeShellScriptBin name
  ''
  set -euo pipefail
  hc-test-fmt
  hc-rust-test
  # hc-app-spec-test
  '';
in
{
 buildInputs = [ script ]

 ++ (pkgs.callPackage ./fmt {
  pkgs = pkgs;
 }).buildInputs
 ;
}
