#[macro_use]
extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};
use std::os::raw::c_char;

#[derive(Serialize, Default, Clone, PartialEq, Deserialize)]
struct TestStruct {
    value: String,
}
#[derive(Serialize, Default, Clone, PartialEq, Deserialize)]
struct OtherTestStruct {
    other: String,
}

#[no_mangle]
pub extern "C" fn test_error_report(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    zome_assert!(stack, false);
    0
}

/// TODO #486 - load and store string from wasm memory
//// Can't do zome_assert!() while testing store_as_json() since it internally uses store_as_json() !
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

// Can't do zome_assert!() while testing store_as_json() since it internally uses store_as_json() !
// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
#[no_mangle]
pub extern "C" fn test_store_as_json_ok(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
    };
    assert_eq!(0, stack.top());
    let res = store_as_json(&mut stack, obj.clone());
    assert_eq!(json!(obj).to_string().len(), stack.top() as usize);
    res.unwrap().encode()
}

// Can't do zome_assert!() while testing store_as_json() since it internally uses store_as_json() !
// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
#[no_mangle]
pub extern "C" fn test_store_as_json_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;
    let maybe_stack = SinglePageStack::from_encoded_allocation(allmost_full_alloc);
    zome_assert!(stack, maybe_stack.is_ok());
    let mut stack = maybe_stack.unwrap();
    let obj = TestStruct {
        value: "fish".to_string(),
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
    };
    assert_eq!(0, stack.top());
    let store_res = store_as_json(&mut stack, obj.clone());
    let ptr = store_res.clone().unwrap().offset() as *mut c_char;
    let load_res: Result<OtherTestStruct, String> = load_json_from_raw(ptr);
    zome_assert!(stack, load_res.is_err());
    let store_err_res = store_as_json(&mut stack, load_res.err().unwrap().clone());
    store_err_res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_json_ok(_: u32) -> u32 {
    let encoded = test_store_as_json_ok(0);
    let mut stack = SinglePageStack::from_encoded_allocation(encoded).unwrap();
    let res: Result<TestStruct, String> = load_json(encoded);
    let res = store_as_json(&mut stack, res.unwrap().clone());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_json_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let res: Result<TestStruct, String> = load_json(1 << 16);
    zome_assert!(stack, res.is_err());
    let res = store_as_json(&mut stack, res.err().unwrap().clone());
    res.unwrap().encode()
}
