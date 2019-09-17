{ pkgs }:
let
 name = "hc-test-fmt";

 # the reason this exists (instead of just hn-rust-fmt-check)
 # is to avoid things that aren't compatible with current version
 # of fmt
 # @todo rip this out when fmt catches up with nightly
 # @see https://github.com/rust-lang/rustfmt/issues/3666
 # @see https://github.com/rust-lang/rustfmt/issues/3685
 script = pkgs.writeShellScriptBin name
 ''
 set -euo pipefail
 echo "checking rust formatting"
 for p in \
  hc \
  holochain_common \
  holochain \
  holochain_conductor_api \
  holochain_conductor_wasm \
  holochain_core_api_c_binding \
  holochain_dna_c_binding \
  hdk \
  hdk-proc-macros \
  holochain_net \
  holochain_dpki \
  holochain_test_bin \
  benchmarks
 do
  echo "checking ''${p}"
  cargo fmt -p $p -- --check
 done
 '';
in
{
 buildInputs = [ script ];
}
