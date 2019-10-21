# For easier testing
set test_p= holochain_dna_c_binding
set test_path= dna_c_binding
set wasm_path=
set wasm_path_2=
.\scripts\windows\do-ci-test.bat

set test_p=holochain_core_types
set test_path=core_types
set wasm_path=
set wasm_path_2=
.\scripts\windows\do-ci-test.bat

set test_p=holochain_wasm_utils
set test_path=wasm_utils
set wasm_path=wasm-test\integration-test
set wasm_path_2=
.\scripts\windows\do-ci-test.bat

set test_p=hdk
set test_path=hdk-rust
set wasm_path=wasm-test
set wasm_path_2=
.\scripts\windows\do-ci-test.bat

set test_p=holochain_conductor_lib
set test_path=conductor_api
set wasm_path=wasm-test
set wasm_path_2=test-bridge-caller
.\scripts\windows\do-ci-test.bat

set test_p=holochain_core
set test_path=core
set wasm_path=src\nucleus\actions\wasm-test
set wasm_path_2=
.\scripts\windows\do-ci-test.bat

set test_p=hc
set test_path=cli
set wasm_path=
set wasm_path_2=
.\scripts\windows\do-ci-test.bat

set test_p=holochain_sodium
set test_path=sodium
set wasm_path=
set wasm_path_2=
.\scripts\windows\do-ci-test.bat

set test_p=holochain_net
set test_path=net
set wasm_path=
set wasm_path_2=
.\scripts\windows\do-ci-test.bat
