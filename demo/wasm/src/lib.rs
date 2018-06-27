//#![feature(wasm_import_memory, custom_attribute)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char, slice};

fn make_internal<'s, T: Deserialize<'s>>(data: *mut c_char) -> T {
    let c_str = unsafe { CStr::from_ptr(data) };
    let actual_str = c_str.to_str().unwrap(); // Don't unwrap ever in real life
    serde_json::from_str(actual_str).unwrap() // OMG you're still doing it! Have you learned nothing?!
}

fn make_external<T: Serialize>(data: *mut c_char, internal: T) -> i32 {
    let json = serde_json::to_string(&internal).unwrap(); //same!

    let bytes = json.as_bytes();
    let len = bytes.len();

    let mem = unsafe { slice::from_raw_parts_mut(data, len) };

    for (i, byte) in bytes.iter().enumerate() {
        mem[i] = *byte as i8;
    }

    len as i32
}

#[no_mangle]
pub extern "C" fn test_dispatch(data: *mut c_char, _params_len: usize) -> i32 {
    let input = make_internal(data);
    let output = test(input);
    make_external(data, output)
}

#[derive(Deserialize, Default)]
struct InputStruct {
    input_int_val: u8,
    input_str_val: String,
}

#[derive(Serialize, Default)]
struct OutputStruct {
    input_int_val_plus2: u8,
    input_str_val_plus_dog: String,
}

fn test(input: InputStruct) -> OutputStruct {
    OutputStruct {
        input_int_val_plus2: input.input_int_val + 2,
        input_str_val_plus_dog: format!("{}.puppy", input.input_str_val),
    }
}
