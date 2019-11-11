{ pkgs }:
let

  test_script = pkgs.writeShellScriptBin "hc-cli-test"
  ''
  ( cd crates/cli && cargo test )
  bats crates/cli/test/hc.bats
  '';

  test_gen_rust_script = pkgs.writeShellScriptBin "hc-cli-generate-rust-test"
  ''
  	hc init /tmp/test_gen_rust
	cd /tmp/test_gen_rust
	hc generate zomes/my_zome rust
	hc test
  '';

  test_gen_rust_proc_script = pkgs.writeShellScriptBin "hc-cli-generate-rust-proc-test"
  ''
  	hc init /tmp/test_gen_rust_proc
	cd /tmp/test_gen_rust_proc
	hc generate zomes/my_zome rust-proc
	hc test
  '';
in

{
 buildInputs = [ 
 	test_script
 	test_gen_rust_script
 	test_gen_rust_proc_script
 ];
}
