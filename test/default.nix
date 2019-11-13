{ pkgs }:
let
  name = "hc-test";

  script = pkgs.writeShellScriptBin name
  ''
  set -euo pipefail
  hc-test-fmt
  hn-rust-clippy
  hc-rust-test
  '';
in
{
 buildInputs = [ script ]

 ++ (pkgs.callPackage ./fmt {
  pkgs = pkgs;
 }).buildInputs
 ;
}
