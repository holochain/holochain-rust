#[macro_use]
extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
use holochain_wasm_utils::holochain_core_types::json::JsonString;
use holochain_wasm_utils::holochain_core_types::json::RawString;

use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};
use std::os::raw::c_char;

#[derive(Serialize, Default, Clone, PartialEq, Deserialize)]
struct TestStruct {
    value: String,
}

impl From<TestStruct> for JsonString {
    fn from(test_struct: TestStruct) -> JsonString {
        JsonString::from(serde_json::to_string(&test_struct).expect("failed to Jsonify TestStruct"))
    }
}

#[derive(Serialize, Default, Clone, PartialEq, Deserialize)]
struct OtherTestStruct {
    other: String,
}

impl From<OtherTestStruct> for JsonString {
    fn from(other_test_struct: OtherTestStruct) -> JsonString {
        JsonString::from(serde_json::to_string(&other_test_struct).expect("failed to Jsonify OtherTestStruct"))
    }
}

#[no_mangle]
pub extern "C" fn test_error_report(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    zome_assert!(stack, false);
    0
}

<<<<<<< HEAD
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

// Can't do zome_assert!() while testing store_json() since it internally uses store_json() !
// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
#[no_mangle]
pub extern "C" fn test_store_json_ok(_: u32) -> u32 {
=======
// General Note:
// Can't do zome_assert!() while testing store_as_json() since
// zome_assert!() internally uses store_as_json().
// So assert!() is used even if we get unhelpful Trap::Unreachable error message.

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
    let res = store_as_json(&mut stack, s);
    assert_eq!(json!(s).to_string().len(), stack.top() as usize);
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_store_as_json_obj_ok(_: u32) -> u32 {
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    let mut stack = SinglePageStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
    };
    assert_eq!(0, stack.top());
    let res = store_json(&mut stack, obj.clone());
    assert_eq!(json!(obj).to_string().len(), stack.top() as usize);
    res.unwrap().encode()
}

<<<<<<< HEAD
// Can't do zome_assert!() while testing store_json() since it internally uses store_json() !
// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
#[no_mangle]
pub extern "C" fn test_store_json_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
=======
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
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    let allmost_full_alloc = 0b1111111111111101_0000000000000010;
    let maybe_stack = SinglePageStack::from_encoded_allocation(allmost_full_alloc);
    assert!(maybe_stack.is_ok());
    let mut stack = maybe_stack.unwrap();
    let obj = TestStruct {
        value: "fish".to_string(),
    };
    let res = store_json(&mut stack, obj.clone());
    assert!(res.is_err());
    res.err().unwrap() as u32
}

#[no_mangle]
pub extern "C" fn test_load_json_from_raw_ok(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let obj = TestStruct {
        value: "fish".to_string(),
    };
    let res = store_json(&mut stack, obj.clone());
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
<<<<<<< HEAD
    assert_eq!(0, stack.top());
    let store_res = store_json(&mut stack, obj.clone());
=======
    let store_res = store_as_json(&mut stack, obj.clone());
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    let ptr = store_res.clone().unwrap().offset() as *mut c_char;
    let load_res: Result<OtherTestStruct, String> = load_json_from_raw(ptr);
    zome_assert!(stack, load_res.is_err());
    let store_err_res = store_json(&mut stack, RawString::from(load_res.err().unwrap().clone()));
    store_err_res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_json_ok(_: u32) -> u32 {
<<<<<<< HEAD
    let encoded = test_store_json_ok(0);
=======
    let encoded = test_store_as_json_obj_ok(0);
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    let mut stack = SinglePageStack::from_encoded_allocation(encoded).unwrap();
    let res: Result<TestStruct, String> = load_json(encoded);
    let res = store_json(&mut stack, res.unwrap().clone());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_json_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let res: Result<TestStruct, String> = load_json(1 << 16);
    zome_assert!(stack, res.is_err());
    let res = store_json(&mut stack, RawString::from(res.err().unwrap().clone()));
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_string_ok(_: u32) -> u32 {
    let encoded = test_store_string_ok(0);
    let mut stack = SinglePageStack::from_encoded_allocation(encoded).unwrap();
    let res: Result<String, String> = load_string(encoded);
    let res = store_string(&mut stack, &res.unwrap());
    res.unwrap().encode()
}

#[no_mangle]
pub extern "C" fn test_load_string_err(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let res: Result<String, String> = load_string(1 << 16);
    zome_assert!(stack, res.is_err());
    let res = store_string(&mut stack, &res.err().unwrap().clone());
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
    let first = store_json_into_encoded_allocation(&mut stack, "first");
    let _second = store_json_into_encoded_allocation(&mut stack, "second");
    first as u32
}

#[no_mangle]
pub extern "C" fn test_stacked_json_obj(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let first = store_json_into_encoded_allocation(&mut stack, TestStruct {
        value: "first".to_string(),
    });
    let _second = store_json_into_encoded_allocation(&mut stack, TestStruct {
        value: "second".to_string(),
    });
    first as u32
}

#[no_mangle]
pub extern "C" fn test_stacked_mix(_: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let _first = store_json_into_encoded_allocation(&mut stack, TestStruct {
        value: "first".to_string(),
    });
    let _second = store_json_into_encoded_allocation(&mut stack, "second");
    let third = store_string_into_encoded_allocation(&mut stack, "third");
    let _fourth = store_json_into_encoded_allocation(&mut stack, "fourth");
    let _fifth = store_json_into_encoded_allocation(&mut stack, TestStruct {
        value: "fifth".to_string(),
    });
    third as u32
}
