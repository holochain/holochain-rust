@echo off
setlocal enabledelayedexpansion

rem KEEP IN SYNC WITH HOLONIX
set nightly-date=nightly-2019-07-14
rustup toolchain install --no-self-update !nightly-date!
rustup default !nightly-date!
rustup target add wasm32-unknown-unknown

IF NOT "!wasm_path!" == "" (
 set manifest-path=!test_path!\!wasm_path!\Cargo.toml
 set target-dir=!test_path!\!wasm_path!\target
 IF NOT "!hc_target_prefix!" == "" (
  set target-dir=!hc_target_prefix!\!target-dir!
 )
 cargo build --manifest-path !manifest-path! --release --target wasm32-unknown-unknown --target-dir !target-dir! --verbose
)

IF NOT "!wasm_path_2!" == "" (
 set manifest-path=!test_path!\!wasm_path_2!\Cargo.toml
 set target-dir=!test_path!\!wasm_path_2!\target
 IF NOT "!hc_target_prefix!" == "" (
  set target-dir=!hc_target_prefix!\!target-dir!
 )
 cargo build --manifest-path !manifest-path%! --release --target wasm32-unknown-unknown --target-dir !target-dir! --verbose
)

set target-dir=!test_path!\target
IF NOT "!hc_target_prefix!" == "" (
 set target-dir=!hc_target_prefix!\!target-dir!
)
cargo test --release -p !test_p! --target-dir !target-dir! --verbose
