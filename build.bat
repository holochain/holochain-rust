cd conductor_api/wasm-test && cargo +nightly-2019-01-24 build --release --target wasm32-unknown-unknown && cd ../../
cd conductor_api/test-bridge-caller && cargo +nightly-2019-01-24 build --release --target wasm32-unknown-unknown && cd ../../
cd hdk-rust/wasm-test && cargo +nightly-2019-01-24 build --release --target wasm32-unknown-unknown && cd ../../
cd wasm_utils/wasm-test/integration-test && cargo +nightly-2019-01-24 build --release --target wasm32-unknown-unknown && cd ../../../
cd core/src/nucleus/actions/wasm-test && cargo +nightly-2019-01-24 build --release --target wasm32-unknown-unknown && cd ../../../../../
cargo +nightly-2019-01-24 build
