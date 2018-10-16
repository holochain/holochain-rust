#[macro_use]
extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use holochain_wasm_utils::{
    memory_allocation::*,
    memory_serialization::*,
};
use std::{os::raw::c_char};

#[derive(Serialize, Default, Clone, PartialEq, Deserialize)]
struct InputTestStruct {
    value: String,
}

#[no_mangle]
pub extern "C" fn test_error_report(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    zome_assert!(stack, false);
    0
}

// Can't do zome_assert!() while testing serialize() since it internally uses serialize() !
// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
#[no_mangle]
pub extern "C" fn test_serialize_ok(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let obj = InputTestStruct { value: "fish".to_string() };
    assert_eq!(0, stack.top());
    let res = serialize(&mut stack, obj.clone());
    assert_eq!(json!(obj).to_string().len(), stack.top() as usize);
    res.unwrap().encode()
}

// Can't do zome_assert!() while testing serialize() since it internally uses serialize() !
#[no_mangle]
pub extern "C" fn test_serialize_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;
    let maybe_stack = SinglePageStack::from_encoded_allocation(allmost_full_alloc);
    zome_assert!(stack, maybe_stack.is_ok());
    let mut stack = maybe_stack.unwrap();
    let obj = InputTestStruct { value: "fish".to_string() };
    let res = serialize(&mut stack, obj.clone());
    assert!(res.is_err());
    res.err().unwrap() as u32
}

#[no_mangle]
pub extern "C" fn test_deserialize_ok(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let obj = InputTestStruct { value: "fish".to_string() };
    let res = serialize(&mut stack, obj.clone());
    let ptr = res.unwrap().offset() as *mut c_char;
    let res = deserialize(ptr);
    assert!(obj == res.unwrap());
    0
}

#[no_mangle]
pub extern "C" fn test_deserialize_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let ser_res = serialize(&mut stack, "some error string");
    let ptr = ser_res.clone().unwrap().offset() as *mut c_char;
    let res: Result<InputTestStruct, String> = deserialize(ptr);
    zome_assert!(stack, res.is_err());
    ser_res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_deserialize_allocation_ok(_: u32) -> u32 {
    let encoded = test_serialize_ok(0);
    let mut stack = SinglePageStack::from_encoded_allocation(encoded).unwrap();
    let res: InputTestStruct = deserialize_allocation(encoded);
    let res = serialize(&mut stack, res.clone());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_try_deserialize_allocation_ok(_: u32) -> u32 {
    let encoded = test_serialize_ok(0);
    let mut stack = SinglePageStack::from_encoded_allocation(encoded).unwrap();
    let res: Result<InputTestStruct, String> = try_deserialize_allocation(encoded);
    let res = serialize(&mut stack, res.unwrap().clone());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_try_deserialize_allocation_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let res: Result<InputTestStruct, String> = try_deserialize_allocation(1 << 16);
    zome_assert!(stack, res.is_err());
    let res = serialize(&mut stack, res.err().unwrap().clone());
    res.unwrap().encode()
}