{ pkgs }:
# simplified version of the c bindings test command in makefile
# hardcodes hc_dna to test rather than looping/scanning like make does
# might want to make this more sophisticated if we end up with many tests
let
 name = "hc-qt-c-bindings-test";

 script = pkgs.writeShellScriptBin name
 ''
 hc-qt-c-bindings-flush
 cargo build -p holochain_dna_c_binding
 ( cd c_binding_tests/hc_dna && qmake -o $@Makefile $@qmake.pro && make );
 ./target/debug/c_binding_tests/hc_dna/test_executable
 '';
in
{
 buildInputs = [ script ];
}
