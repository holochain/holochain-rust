extern {
    fn print(i:i32);
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

    return 0;//a * a;
}

//pub extern "C" fn _call(a: i32) -> i32 {
//    unsafe {
//        print(123);
//    }
//
//    return a * a;
//}
