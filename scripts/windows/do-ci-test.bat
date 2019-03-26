IF NOT [%wasm_path] == [] (
 cargo build --manifest-path "%test_path%%wasm_path%Cargo.toml" --release --target wasm32-unknown-unknown --target-dir "%hc_target_prefix%%test_path%%wasm_path%target"
)

IF NOT [%wasm_path_2] == [] (
 cargo build --manifest-path "%test_path%%wasm_path_2%Cargo.toml" --release --target wasm32-unknown-unknown --target-dir "%hc_target_prefix%%test_path%%wasm_path_2%target"
)

cargo test --release -p "%test_p%" --target-dir "%hc_target_prefix%%test_path%target"
