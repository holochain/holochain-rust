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

use holochain_wasm_utils::{
    memory_allocation::*, memory_serialization::*,
    holochain_core_types::error::HolochainError,
};
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
pub extern "C" fn test_error_report(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    zome_assert!(stack, false);
    0
}

// TODO #486 - load and store string from wasm memory
//// Can't do zome_assert!() while testing store_json() since it internally uses store_json() !
//// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
//#[no_mangle]
//pub extern "C" fn test_store_string_ok(_: u32) -> u32 {
//    let mut stack = SinglePageStack::default();
//    let s = "some string";
//    assert_eq!(0, stack.top());
//    let res = store_string(&mut stack, s);
//    //assert_eq!(obj.len(), stack.top() as usize);
//    //res.unwrap().encode()
//    0
//}

#[no_mangle]
pub extern "C" fn test_store_string_ok(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let s = "fish";
    assert_eq!(0, stack.top());
    let res = store_string(&mut stack, s);
    assert_eq!(s.len(), stack.top() as usize);
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_store_as_json_str_ok(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let s = "fish";
    assert_eq!(0, stack.top());

    let res = store_as_json(&mut stack, RawString::from(s));
    assert_eq!(json!(s).to_string().len(), stack.top() as usize);

    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_store_as_json_obj_ok(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    assert_eq!(0, stack.top());
    let res = store_as_json(&mut stack, obj.clone());
    assert_eq!(json!(obj).to_string().len(), stack.top() as usize);
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_store_string_err(_: u32) -> u32 {
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;
    let maybe_stack = SinglePageStack::from_encoded_allocation(allmost_full_alloc);
    assert!(maybe_stack.is_ok());
    let mut stack = maybe_stack.unwrap();
    let s = "fish";
    let res = store_string(&mut stack, s);
    assert!(res.is_err());
    res.err().unwrap() as u32
}

#[no_mangle]
pub extern "C" fn test_store_as_json_err(_: u32) -> u32 {
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;
    let maybe_stack = SinglePageStack::from_encoded_allocation(allmost_full_alloc);
    assert!(maybe_stack.is_ok());
    let mut stack = maybe_stack.unwrap();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    let res = store_as_json(&mut stack, obj.clone());
    assert!(res.is_err());
    res.err().unwrap() as u32
}

#[no_mangle]
pub extern "C" fn test_load_json_from_raw_ok(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    let res = store_as_json(&mut stack, obj.clone());
    let ptr = res.unwrap().offset() as *mut c_char;
    let res = load_json_from_raw(ptr);
    assert!(obj == res.unwrap());
    0
}

#[no_mangle]
pub extern "C" fn test_load_json_from_raw_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    assert_eq!(0, stack.top());
    let store_res = store_as_json(&mut stack, obj.clone());
    let ptr = store_res.clone().unwrap().offset() as *mut c_char;
    let load_res: Result<OtherTestStruct, HolochainError> = load_json_from_raw(ptr);
    zome_assert!(stack, load_res.is_err());
    let store_err_res = store_as_json(&mut stack, load_res.err().unwrap().to_string());
    store_err_res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_json_ok(_: u32) -> u32 {
    let encoded = test_store_as_json_obj_ok(0);
    let mut stack = SinglePageStack::from_encoded_allocation(encoded).unwrap();
    let res: Result<TestStruct, HolochainError> = load_json(encoded);
    let res = store_as_json(&mut stack, res.unwrap().clone());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_json_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let res: Result<TestStruct, HolochainError> = load_json(1 << 16);
    zome_assert!(stack, res.is_err());
    let res = store_as_json(&mut stack, res);
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_string_ok(_: u32) -> u32 {
    let encoded = test_store_string_ok(0);
    let mut stack = SinglePageStack::from_encoded_allocation(encoded).unwrap();
    let res = load_string(encoded);
    let res = store_string(&mut stack, &res.unwrap());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_string_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let res = load_string(1 << 16);
    zome_assert!(stack, res.is_err());
    let res = store_string(&mut stack, &res.err().unwrap().to_string());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_stacked_strings(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let first = store_string_into_encoded_allocation(&mut stack, "first");
    let _second = store_string_into_encoded_allocation(&mut stack, "second");
    first as u32
}

#[no_mangle]
pub extern "C" fn test_stacked_json_str(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let first = store_as_json_into_encoded_allocation(&mut stack, "first");
    let _second = store_as_json_into_encoded_allocation(&mut stack, "second");
    first as u32
}

#[no_mangle]
pub extern "C" fn test_stacked_json_obj(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let first = store_as_json_into_encoded_allocation(&mut stack, TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    });
    let _second = store_as_json_into_encoded_allocation(&mut stack, TestStruct {
        value: "second".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    });
    first as u32
}

#[no_mangle]
pub extern "C" fn test_stacked_mix(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let _first = store_as_json_into_encoded_allocation(&mut stack, TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    });
    let _second = store_as_json_into_encoded_allocation(&mut stack, "second");
    let third = store_string_into_encoded_allocation(&mut stack, "third");
    let _fourth = store_as_json_into_encoded_allocation(&mut stack, "fourth");
    let _fifth = store_as_json_into_encoded_allocation(&mut stack, TestStruct {
        value: "fifth".to_string(),
        list: vec!["fifthlist".to_string()],
    });
    third as u32
}
