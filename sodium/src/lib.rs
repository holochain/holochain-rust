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
macro_rules! rptr {
    ($name: ident) => {
        $name.as_mut_ptr() as *mut libc::c_void
    };
}

pub mod random;
pub mod secbuf;
pub mod util;
