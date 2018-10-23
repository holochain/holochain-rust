cd core/src/nucleus/wasm-test && cargo +nightly-2018-10-12 build --release --target wasm32-unknown-unknown && cd ../../../../
cd core_api/wasm-test/round_trip && cargo +nightly-2018-10-12 build --release --target wasm32-unknown-unknown && cd ../../../
cd core_api/wasm-test/commit && cargo +nightly-2018-10-12 build --release --target wasm32-unknown-unknown && cd ../../../
cd hdk-rust/wasm-test && cargo +nightly-2018-10-12 build --release --target wasm32-unknown-unknown && cd ../../
cd wasm_utils/wasm-test/integration-test && cargo +nightly-2018-10-12 build --release --target wasm32-unknown-unknown && cd ../../../
cd core/src/nucleus/actions/wasm-test && cargo +nightly-2018-10-12 build --release --target wasm32-unknown-unknown && cd ../../../../../
cargo +nightly-2018-10-12 build
