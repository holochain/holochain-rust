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
 echo "checking rust formatting";
 cd crates;
 ls;
 for p in \
  cli \
  ../core \
  ../core_types \
  ../holochain \
  ../conductor_lib \
  ../holochain_wasm \
  ../hdk \
  ../hdk-v2 \
  ../net \
  ../dpki \
  ../sim2h \
  ../sim2h_server \
  ../../common \
  ../benchmarks \
  ../test_utils \

 do
  echo "checking ''${p}"
  cd $p && cargo fmt -- --check
 done
 '';
in
{
 buildInputs = [ script ];
}
