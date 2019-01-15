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
use std::os::raw::c_char;

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
    assert_eq!(0, stack.top());
    let res = stack.write_string(s);
    assert_eq!(s.len(), usize::from(stack.top()));
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_store_as_json_str_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let s = "fish";
    assert_eq!(0, stack.top());

    let res = stack.write_json(RawString::from(s));
    assert_eq!(json!(s).to_string().len(), usize::from(stack.top()));

    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_store_as_json_obj_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    assert_eq!(0, stack.top());
    let res = stack.write_json(obj.clone());
    assert_eq!(json!(obj).to_string().len(), stack.top() as usize);
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_store_string_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;
    let maybe_stack = WasmStack::from_encoded_allocation(allmost_full_alloc);
    assert!(maybe_stack.is_ok());
    let mut stack = maybe_stack.unwrap();
    let s = "fish";
    let res = stack.write_string(s);
    assert!(res.is_err());
    res.err().unwrap() as RibosomeEncodingBits
}

#[no_mangle]
pub extern "C" fn test_store_as_json_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;
    let maybe_stack = WasmStack::from_encoded_allocation(allmost_full_alloc);
    assert!(maybe_stack.is_ok());
    let mut stack = maybe_stack.unwrap();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    let res = stack.write_json(obj.clone());
    assert!(res.is_err());
    res.err().unwrap() as RibosomeEncodingBits
}

#[no_mangle]
pub extern "C" fn test_load_json_from_raw_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    let res = stack.write_json(obj.clone());
    let ptr = res.unwrap().offset() as *mut c_char;
    let res = load_json_from_raw(ptr);
    assert!(obj == res.unwrap());

    RibosomeReturnCode::Success.into()
}

#[no_mangle]
pub extern "C" fn test_load_json_from_raw_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    assert_eq!(0, stack.top());
    let store_res = stack.write_json(obj.clone());
    let ptr = store_res.clone().unwrap().offset() as *mut c_char;
    let load_res: Result<OtherTestStruct, HolochainError> = load_json_from_raw(ptr);
    zome_assert!(stack, load_res.is_err());
    let store_err_res = stack.write_json(load_res.err().unwrap().to_string());
    store_err_res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_json_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let encoded = test_store_as_json_obj_ok(0);
    let mut stack = WasmStack::from_encoded_allocation(encoded).unwrap();
    let res: Result<TestStruct, HolochainError> = load_json(encoded);
    let res = stack.write_json(res.unwrap().clone());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_json_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let res: Result<TestStruct, HolochainError> = load_json(1 << 16);
    zome_assert!(stack, res.is_err());
    let res = stack.write_json(res);
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_string_ok(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let encoded = test_store_string_ok(0);
    let mut stack = WasmStack::from_encoded_allocation(encoded).unwrap();
    let res = load_string(encoded);
    let res = stack.write_string(&res.unwrap());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_string_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let res = load_string(1 << 16);
    zome_assert!(stack, res.is_err());
    let res = stack.write_string(&res.err().unwrap().to_string());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_stacked_strings(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let first = stack.write_string("first");
    let _second = stack.write_string("second");
    first as RibosomeEncodingBits
}

#[no_mangle]
pub extern "C" fn test_stacked_json_str(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let first = stack.write_json("first");
    let _second = stack.write_json("second");
    first as RibosomeEncodingBits
}

#[no_mangle]
pub extern "C" fn test_stacked_json_obj(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    let first = stack.write_json(testStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    });
    let _second = stack.write_json(TestStruct {
        value: "second".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    });
    first as RibosomeEncodingBits
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
    third as RibosomeEncodingBits
}
