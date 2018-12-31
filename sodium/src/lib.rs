extern crate rust_sodium_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;

lazy_static! {
    /// we only need to call sodium_init once
    static ref INIT: bool = {
        unsafe {
            rust_sodium_sys::sodium_init();
        }
        true
    };
}

/// make sure sodium_init is called
pub fn check_init() {
    assert_eq!(true, *INIT);
}

/// make invoking ffi functions taking SecBuf references more readable
macro_rules! raw_ptr_void {
    ($name: ident) => {
        $name.as_mut_ptr() as *mut libc::c_void
    };
}

/// make invoking ffi functions taking SecBuf references more readable
macro_rules! raw_ptr_char {
    ($name: ident) => {
        $name.as_mut_ptr() as *mut libc::c_uchar
    };
}

/// make invoking ffi functions taking SecBuf references more readable
macro_rules! raw_ptr_char_immut {
    ($name: ident) => {
        $name.as_ptr() as *const libc::c_uchar
    };
}

/// make invoking ffi functions taking SecBuf references more readable
macro_rules! raw_ptr_ichar_immut {
    ($name: ident) => {
        $name.as_ptr() as *const libc::c_char
    };
}
pub mod random;
pub mod secbuf;
pub mod util;
pub mod sign;
pub mod hash;
pub mod aead;
pub mod kdf;
pub mod pwhash;
pub mod kx;
pub mod error;
