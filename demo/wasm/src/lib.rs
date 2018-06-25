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

use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn test(data: *mut c_char, input_len: usize) -> i32 {
    /* params = unserialze_params(data);

    #do what ever
    return_value = do_whatever();

    serialize_to_memory(return_value);
*/

    unsafe {
        *data.offset(2) = 31;
    }
    5
}

//pub extern "C" fn _call(a: i32) -> i32 {
//    unsafe {
//        print(123);
//    }
//
//    return a * a;
//}
