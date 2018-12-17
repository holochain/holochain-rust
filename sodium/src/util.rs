//! This module provides access to libsodium utility and memory functions

use super::check_init;

use super::secbuf::SecBuf;

/// Zero all memory within the provided SecBuf
pub fn zero(b: &mut SecBuf) {
    check_init();
    unsafe {
        let mut b = b.write_lock();
        rust_sodium_sys::sodium_memzero(raw_ptr_void!(b), b.len());
    }
}

/// Increments all memory within the provided SecBuf by 1
pub fn increment(b: &mut SecBuf) {
    check_init();
    unsafe{
        let mut b = b.write_lock();
        rust_sodium_sys::sodium_increment(raw_ptr_char!(b), b.len());
    }
}

/// Compares the Two SecBuf
/// Return :
/// | if a > b; return 1
/// | if a < b; return -1
/// | if a == b; return 0
pub fn compare(a: &mut SecBuf,b: &mut SecBuf) -> i32 {
    check_init();
    unsafe{
        let mut a = a.write_lock();
        let mut b = b.write_lock();
        rust_sodium_sys::sodium_compare(raw_ptr_char!(a),raw_ptr_char!(b), a.len())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_zero_buffer() {
        let mut b = SecBuf::with_insecure(1);

        {
            let mut b = b.write_lock();
            b[0] = 42;
        }

        zero(&mut b);

        {
            let b = b.read_lock();
            assert_eq!(0, b[0]);
        }
    }
    #[test]
    fn it_should_increment_buffer() {
        let mut b = SecBuf::with_insecure(1);

        {
            let mut b = b.write_lock();
            b[0] = 42;
        }

        increment(&mut b);

        {
            let b = b.read_lock();
            assert_eq!(43, b[0]);
        }
    }
    #[test]
    fn it_should_compare_buffer() {
        let mut a = SecBuf::with_insecure(1);
        {
            let mut a = a.write_lock();
            a[0] = 50;
        }
        let mut b = SecBuf::with_insecure(1);
        {
            let mut b = b.write_lock();
            b[0] = 45;
        }
        let mut c = SecBuf::with_insecure(1);
        {
            let mut c = c.write_lock();
            c[0] = 45;
        }

        let val_1 = compare(&mut a,&mut b);
        let val_2 = compare(&mut b,&mut a);
        let val_3 = compare(&mut b,&mut c);
        {
            assert_eq!(1, val_1);
            assert_eq!(-1, val_2);
            assert_eq!(0, val_3);
        }
    }
}
