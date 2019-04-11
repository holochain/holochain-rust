for /f %%i in ('Join-Path %test_path% %wasm_path% Cargo.toml') do set foo=%i
echo %foo%

IF NOT "%wasm_path%" == "" (
 for /f %%i in ('Join-Path %test_path% %wasm_path% Cargo.toml') do set manifest-path=%i
 for /f %%i in ('Join-Path %hc_target_prefix% %test_path% %wasm_path% target') do set target-dir=%i
 cargo build --manifest-path "%manifest-path%" --release --target wasm32-unknown-unknown --target-dir "%target-dir%"
)

IF NOT "%wasm_path_2%" == "" (
 for /f %%i in ('Join-Path %test_path% %wasm_path_2% Cargo.toml') do set manifest-path=%i
 for /f %%i in ('Join-Path %hc_target_prefix% %test_path% %wasm_path_2% target') do set target-dir=%i
 cargo build --manifest-path "%manifest-path%" --release --target wasm32-unknown-unknown --target-dir "%target-dir%"
)

for /f %%i in ('Join-Path %hc_target_prefix% %test_path% target') do set target-dir=%i
cargo test --release -p "%test_p%" --target-dir "%target-dir%"
