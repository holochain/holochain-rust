foo=Join-Path %test_path% %wasm_path% Cargo.toml )
echo %foo%

IF NOT "%wasm_path%" == "" (
 manifest-path=Join-Path %test_path% %wasm_path% Cargo.toml
 target-dir=Join-Path %hc_target_prefix% %test_path% %wasm_path% target
 cargo build --manifest-path "%manifest-path%" --release --target wasm32-unknown-unknown --target-dir "%target-dir%"
)

IF NOT "%wasm_path_2%" == "" (
 manifest-path=Join-Path %test_path% %wasm_path_2% Cargo.toml
 target-dir=Join-Path %hc_target_prefix% %test_path% %wasm_path_2% target
 cargo build --manifest-path "%manifest-path%" --release --target wasm32-unknown-unknown --target-dir "%target-dir%"
)

target-dir=Join-Path %hc_target_prefix% %test_path% target
cargo test --release -p "%test_p%" --target-dir "%target-dir%"
