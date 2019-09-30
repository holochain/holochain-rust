{ pkgs }:
let
  name = "hc-app-spec-test-proc";

  # the test subdirectory gets blown away every run
  # thats why this is here
  script = pkgs.writeShellScriptBin name ''
  hc-cli-install \
  && hc-conductor-install \
  && (cd app_spec_proc_macro && ./build_and_test.sh);
  '';
in
{
 buildInputs = [ script ];
}
