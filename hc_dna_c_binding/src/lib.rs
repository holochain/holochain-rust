//! This crate is an ffi wrapper to provide a c-compatible dna library.
//!
//! Remember to free all dna objects and returned strings.
//!
//! See the associated Qt unit tests in the c_binding_tests directory.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::catch_unwind;

extern crate hc_dna;

use hc_dna::Dna;

#[no_mangle]
pub extern "C" fn hc_dna_create() -> *mut Dna {
    match catch_unwind(|| Box::into_raw(Box::new(Dna::new()))) {
        Ok(r) => r,
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn hc_dna_create_from_json(buf: *const c_char) -> *mut Dna {
    match catch_unwind(|| {
        let json = unsafe { CStr::from_ptr(buf).to_string_lossy().into_owned() };
        let dna = match Dna::new_from_json(&json) {
            Ok(d) => d,
            Err(_) => return std::ptr::null_mut(),
        };

        Box::into_raw(Box::new(dna))
    }) {
        Ok(r) => r,
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn hc_dna_free(ptr: *mut Dna) {
    catch_unwind(|| {
        if ptr.is_null() {
            return;
        }
        unsafe {
            Box::from_raw(ptr);
        }
    }).unwrap_or(());
}

#[no_mangle]
pub extern "C" fn hc_dna_to_json(ptr: *const Dna) -> *mut c_char {
    match catch_unwind(|| {
        let dna = unsafe {
            assert!(!ptr.is_null());
            &*ptr
        };

        let json = match dna.to_json() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        let json = match CString::new(json) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        json.into_raw()
    }) {
        Ok(r) => r,
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn hc_dna_string_free(s: *mut c_char) {
    catch_unwind(|| {
        if s.is_null() {
            return;
        }
        unsafe {
            CString::from_raw(s);
        }
    }).unwrap_or(());
}

#[no_mangle]
pub extern "C" fn hc_dna_get_dna_spec_version(ptr: *const Dna) -> *mut c_char {
    match catch_unwind(|| {
        let dna = unsafe {
            assert!(!ptr.is_null());
            &*ptr
        };
        let version = dna.dna_spec_version.clone();

        let res = match CString::new(version) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        res.into_raw()
    }) {
        Ok(r) => r,
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_and_deserialize() {
        let dna = hc_dna_create();
        let dna_json_raw = hc_dna_to_json(dna);
        hc_dna_free(dna);

        let dna2 = hc_dna_create_from_json(dna_json_raw);
        hc_dna_string_free(dna_json_raw);

        let version_raw = hc_dna_get_dna_spec_version(dna2);
        let version_str = unsafe { CStr::from_ptr(version_raw).to_string_lossy().into_owned() };
        assert_eq!(version_str, String::from("2.0"));
        hc_dna_string_free(version_raw);

        hc_dna_free(dna2);
    }
}
