cd container_api/wasm-test && cargo +nightly-2018-12-26 build --release --target wasm32-unknown-unknown && cd ../../
cd hdk-rust/wasm-test && cargo +nightly-2018-12-26 build --release --target wasm32-unknown-unknown && cd ../../
cd wasm_utils/wasm-test/integration-test && cargo +nightly-2018-12-26 build --release --target wasm32-unknown-unknown && cd ../../../
cd core/src/nucleus/actions/wasm-test && cargo +nightly-2018-12-26 build --release --target wasm32-unknown-unknown && cd ../../../../../
cargo +nightly-2018-12-26 build
