#![feature(wasm_import_memory, custom_attribute)]
#![wasm_import_memory]

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
use std::ffi::CString;
use std::os::raw::c_char;

fn my_string_safe(i: *mut c_char) -> String {
    unsafe { CStr::from_ptr(i).to_string_lossy().into_owned() }
}

#[no_mangle]
pub extern "C" fn test(data: *mut c_char, params_len: usize) -> i32 {
    /* params = unserialze_params(data);

    #do what ever
    return_value = do_whatever();

    serialize_to_memory(return_value);
     */

    //    let mut s = my_string_safe(data);
    let s = "fish".to_string();
    let bytes = s.as_bytes();

    let len = bytes.len();

    unsafe {
        for i in 0..len {
            *data.offset((i + params_len) as isize) = bytes[i as usize] as i8;
        }
    }

    len as i32 + params_len as i32
}

//pub extern "C" fn _call(a: i32) -> i32 {
//    unsafe {
//        print(123);
//    }
//
//    return a * a;
//}
