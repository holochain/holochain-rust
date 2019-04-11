@echo off
setlocal enabledelayedexpansion

echo !test_path!
echo !wasm_path!

IF NOT "!wasm_path!" == "" (
 set manifest-path=!test_path!!wasm_path!Cargo.toml
 set target-dir=!test_path!!wasm_path!target
 IF NOT "!hc_target_prefix!" == "" (
  set target-dir=!hc_target_prefix!!target-dir!
 )
 echo "x: !manifest-path!"
 echo "y: !target-dir!"
 cargo build --manifest-path !manifest-path! --release --target wasm32-unknown-unknown --target-dir !target-dir!
)

IF NOT "!wasm_path_2!" == "" (
 set manifest-path=!test_path!\!wasm_path_2!\Cargo.toml
 set target-dir=!test_path!\!wasm_path_2!\target
 IF NOT "!hc_target_prefix!" == "" (
  set target-dir=!hc_target_prefix!\!target-dir!
 )
 echo !manifest-path!
 echo !target-dir!
 cargo build --manifest-path !manifest-path%! --release --target wasm32-unknown-unknown --target-dir !target-dir!
)

set target-dir=!test_path!\target
IF NOT "!hc_target_prefix!" == "" (
 set target-dir=!hc_target_prefix!\!target-dir!
)
echo !target-dir!
cargo test --release -p !test_p! --target-dir !target-dir!
