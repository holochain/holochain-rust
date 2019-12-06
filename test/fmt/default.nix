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
 __fmtexit=0
 for p in crates/*; do
  echo "checking ''${p}"
  if ! ( cd "$p" && cargo fmt -- --check ); then __fmtexit=1; fi
 done
 exit ''${__fmtexit}
 '';
in
{
 buildInputs = [ script ];
}
