{ pkgs }:
let
 name = "hc-fmt-all";

 #due to changes in the repo structure, 
 # I thought it would be nice for hn-rust to fmt a single crate
 # while hc-fmt-all formats every crate

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
  ../../common \
  ../benchmarks \
  ../test_utils \
  ../logging
 do
  echo "checking ''${p}"
  cd $p && hn-rust-fmt
 done
 '';
in
{
 buildInputs = [ script ];
}
