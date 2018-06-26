#![feature(wasm_import_memory, custom_attribute)]
#![wasm_import_memory]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern "C" {
    fn print(i: i32);
}

#[no_mangle]
pub extern "C" fn _call(input_data: *mut u8, input_len: usize) -> i32 {
    unsafe {
        print(input_len as i32);
        for i in 0..input_len {
            //print(i as i32);
            //print(888);
            print(*input_data.offset(i as isize) as i32);
        }
    }

    return 0; //a * a;
}

use std::ffi::CStr;
use std::os::raw::c_char;

fn make_string(i: *mut c_char) -> String {
    let s = unsafe { CStr::from_ptr(i) };
    let mut x = 1;
    s.to_string_lossy().into_owned()
}

#[no_mangle]
pub extern "C" fn test_dispatch(data: *mut c_char, params_len: usize) -> i32 {
    let param_as_string = make_string(data);

    let output = test(input);
}

#[derive(Deserialize)]
struct InputStruct {
    input_int_val: u8,
    input_str_val: String,
}

#[derive(Serialize)]
struct OutputStruct {
    input_int_val_plus2: u8,
    input_str_val_plus_dog: String,
}

fn test(input: InputStruct) -> OutputStruct {}
