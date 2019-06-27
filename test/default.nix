{ pkgs }:
let
  name = "hc-test";

  script = pkgs.writeShellScriptBin name
  ''
  hn-rust-fmt-check \
  && hc-qt-c-bindings-test \
  && hc-rust-test \
  && hc-app-spec-test
  '';
in
{
 buildInputs = [ script ];
}
