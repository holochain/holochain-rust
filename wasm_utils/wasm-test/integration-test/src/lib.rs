#![feature(try_from)]
#[macro_use]
extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate holochain_core_types_derive;
use holochain_wasm_utils::holochain_core_types::json::JsonString;
use holochain_wasm_utils::holochain_core_types::json::RawString;
use holochain_wasm_utils::memory::stack::WasmStack;

use holochain_wasm_utils::{
    holochain_core_types::error::HolochainError,
};
use holochain_wasm_utils::holochain_core_types::error::RibosomeEncodingBits;
use holochain_wasm_utils::holochain_core_types::error::RibosomeReturnCode;
use std::convert::TryFrom;
use holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result;
use holochain_wasm_utils::memory::ribosome::allocation_from_ribosome_encoding;
use holochain_wasm_utils::memory::MemoryInt;
use holochain_wasm_utils::memory::ribosome::load_ribosome_encoded_json;

#[derive(Serialize, Default, Clone, PartialEq, Deserialize, Debug, DefaultJson)]
struct TestStruct {
    value: String,
    list: Vec<String>,
}

#[derive(Serialize, Default, Clone, PartialEq, Deserialize, Debug, DefaultJson)]
struct OtherTestStruct {
    other: String,
    list: Vec<String>,
}

#[no_mangle]
pub extern "C" fn test_error_report(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    zome_assert!(stack, false);
    RibosomeReturnCode::Success.into()
}

// TODO #486 - load and store string from wasm memory
//// Can't do zome_assert!() while testing write_json() since it internally uses write_json() !
//// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
//#[no_mangle]
//pub extern "C" fn test_store_string_ok(_: u32) -> u32 {
//    let mut stack = WasmStack::default();
//    let s = "some string";
//    assert_eq!(0, stack.top());
//    let res = stack.write_string(s);
//    //assert_eq!(obj.len(), stack.top() as usize);
//    //res.unwrap().encode()
//    0
//}

#[no_mangle]
pub extern "C" fn test_store_string_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let s = "fish";
    assert_eq!(0, MemoryInt::from(stack.top()));
    let res = stack.write_string(s);
    assert_eq!(usize::from(s.len()), usize::from(stack.top()));

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_store_as_json_str_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let s = "fish";
    assert_eq!(0, MemoryInt::from(stack.top()));

    let res = stack.write_json(RawString::from(s));
    assert_eq!(
        usize::from(json!(s).to_string().len()),
        usize::from(stack.top()),
    );

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_store_as_json_obj_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    assert_eq!(0, MemoryInt::from(stack.top()));
    let res = stack.write_json(obj.clone());
    assert_eq!(
        usize::from(json!(obj).to_string().len()),
        usize::from(stack.top()),
    );

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_store_string_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;

    let allocation = allocation_from_ribosome_encoding(allmost_full_alloc).unwrap();
    let mut stack = WasmStack::try_from(allocation).unwrap();

    let s = "fish";
    let res = stack.write_string(s);
    assert!(res.is_err());

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_store_as_json_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;
    let allocation = allocation_from_ribosome_encoding(allmost_full_alloc).unwrap();
    let mut stack = WasmStack::try_from(allocation).unwrap();

    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    let res = stack.write_json(obj.clone());

    assert!(res.is_err());

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_load_json_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let encoded = test_store_as_json_obj_ok(0);

    let allocation = allocation_from_ribosome_encoding(encoded).unwrap();
    let mut stack = WasmStack::try_from(allocation).unwrap();

    let res: Result<TestStruct, HolochainError> = load_ribosome_encoded_json(encoded);
    let res = stack.write_json(res.unwrap().clone());

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_load_json_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let encoded = 1 << 16;

    let mut stack = WasmStack::default();

    let res: Result<TestStruct, HolochainError> = load_ribosome_encoded_json(encoded);
    zome_assert!(stack, res.is_err());
    let res = stack.write_json(res);

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_load_string_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let encoded = test_store_string_ok(0);

    let allocation = allocation_from_ribosome_encoding(encoded).unwrap();
    let mut stack = WasmStack::try_from(allocation).unwrap();

    let res = allocation.read_to_string();
    let res = stack.write_string(&res);

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_load_string_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let encoded = 1 << 16;
    let allocation = allocation_from_ribosome_encoding(encoded).unwrap();

    let mut stack = WasmStack::try_from(allocation).unwrap();

    let s = allocation.read_to_string();
    let res = stack.write_string(&s);

    return_code_for_allocation_result(res).into()
}

#[no_mangle]
pub extern "C" fn test_stacked_strings(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let first = stack.write_string("first");
    let _second = stack.write_string("second");

    return_code_for_allocation_result(first).into()
}

#[no_mangle]
pub extern "C" fn test_stacked_json_str(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let first = stack.write_json("first");
    let _second = stack.write_json("second");

    return_code_for_allocation_result(first).into()
}

#[no_mangle]
pub extern "C" fn test_stacked_json_obj(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let first = stack.write_json(TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    });
    let _second = stack.write_json(TestStruct {
        value: "second".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    });

    return_code_for_allocation_result(first).into()
}

#[no_mangle]
pub extern "C" fn test_stacked_mix(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let _first = stack.write_json(TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    });
    let _second = stack.write_json("second");
    let third = stack.write_json("third");
    let _fourth = stack.write_json("fourth");
    let _fifth = stack.write_json(TestStruct {
        value: "fifth".to_string(),
        list: vec!["fifthlist".to_string()],
    });

    return_code_for_allocation_result(third).into()
}
