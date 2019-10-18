cd crates/conductor_lib/wasm-test && cargo +nightly-2019-07-14 build --release --target wasm32-unknown-unknown && cd ../../
cd crates/conductor_lib/test-bridge-caller && cargo +nightly-2019-07-14 build --release --target wasm32-unknown-unknown && cd ../../
cd crates/hdk/wasm-test && cargo +nightly-2019-07-14 build --release --target wasm32-unknown-unknown && cd ../../
cd crates/wasm_utils/wasm-test/integration-test && cargo +nightly-2019-07-14 build --release --target wasm32-unknown-unknown && cd ../../../
cd crates/core/src/nucleus/actions/wasm-test && cargo +nightly-2019-07-14 build --release --target wasm32-unknown-unknown && cd ../../../../../
cargo +nightly-2019-07-14 build
