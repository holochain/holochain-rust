extern {
    fn print(i:i32);
}

#[no_mangle]
pub extern "C" fn test_print() -> i32 {
    unsafe {
        print(1337);
    }

    return 0;
}
