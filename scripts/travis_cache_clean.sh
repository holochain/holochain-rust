rm -rf ./target/.rustc_info.json

rm -rf ./target/debug/holochain*
rm -rf ./target/debug/libholochain*
rm -rf ./target/debug/libtest_utils*
rm -rf ./target/debug/test_utils*
rm -rf ./target/debug/integration_test*
rm -rf ./target/debug/hdk*
rm -rf ./target/debug/test_bin_ipc*
rm -rf ./target/debug/test_bin_mock_net*
rm -rf ./target/debug/hc*

rm -rf ./target/debug/.fingerprint/holochain*/
rm -rf ./target/debug/.fingerprint/test_utils*/

# should match debug above, but under deps
rm -rf ./target/debug/deps/holochain*
rm -rf ./target/debug/deps/libholochain*
rm -rf ./target/debug/deps/libtest_utils*
rm -rf ./target/debug/deps/test_utils*
rm -rf ./target/debug/deps/integration_test*
rm -rf ./target/debug/deps/hdk*
rm -rf ./target/debug/deps/test_bin_ipc*
rm -rf ./target/debug/deps/test_bin_mock_net*
rm -rf ./target/debug/deps/hc*

# incremental is just our new stuff so should not cache
rm -rf ./target/debug/incremental/*

# heavy wasms
rm -rf ./core/src/nucleus/actions/wasm-test/target/wasm32-unknown-unknown/debug/deps/libholochain*
