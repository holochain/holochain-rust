//! This crate is an ffi wrapper to provide a c-compatible dna library.
//!
//! Remember to free all dna objects and returned strings.
//!
//! See the associated Qt unit tests in the c_binding_tests directory.

extern crate holochain_dna;

use holochain_dna::Dna;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    panic::catch_unwind,
};

#[no_mangle]
pub extern "C" fn holochain_dna_create() -> *mut Dna {
    match catch_unwind(|| Box::into_raw(Box::new(Dna::new()))) {
        Ok(r) => r,
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn holochain_dna_create_from_json(buf: *const c_char) -> *mut Dna {
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
pub extern "C" fn holochain_dna_free(ptr: *mut Dna) {
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
pub extern "C" fn holochain_dna_to_json(ptr: *const Dna) -> *mut c_char {
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
pub extern "C" fn holochain_dna_string_free(s: *mut c_char) {
    catch_unwind(|| {
        if s.is_null() {
            return;
        }
        unsafe {
            CString::from_raw(s);
        }
    }).unwrap_or(());
}

/// This macro takes care boilerplate for getting string accessors over ffi.
/// This is not exported, it is only meant to be used internally.
macro_rules! _xa_str {
    ($struct:ident, $prop:ident, $getname:ident, $setname:ident) => {
        #[no_mangle]
        pub extern "C" fn $getname(ptr: *const $struct) -> *mut c_char {
            match catch_unwind(|| {
                let arg = unsafe {
                    if ptr.is_null() {
                        return std::ptr::null_mut();
                    }
                    &*ptr
                };

                let res = arg.$prop.clone();

                let res = match CString::new(res) {
                    Ok(s) => s,
                    Err(_) => return std::ptr::null_mut(),
                };

                res.into_raw()
            }) {
                Ok(r) => r,
                Err(_) => std::ptr::null_mut(),
            }
        }

        #[no_mangle]
        pub extern "C" fn $setname(ptr: *mut $struct, val: *const c_char) {
            catch_unwind(|| {
                let arg = unsafe {
                    if ptr.is_null() {
                        return;
                    }
                    &mut *ptr
                };
                let val = unsafe { CStr::from_ptr(val).to_string_lossy().into_owned() };
                arg.$prop = val;
            }).unwrap_or(());
        }
    };
}

_xa_str!(Dna, name, holochain_dna_get_name, holochain_dna_set_name);

_xa_str!(
    Dna,
    description,
    holochain_dna_get_description,
    holochain_dna_set_description
);

_xa_str!(
    Dna,
    version,
    holochain_dna_get_version,
    holochain_dna_set_version
);

_xa_str!(Dna, uuid, holochain_dna_get_uuid, holochain_dna_set_uuid);

_xa_str!(
    Dna,
    dna_spec_version,
    holochain_dna_get_dna_spec_version,
    holochain_dna_set_dna_spec_version
);

#[cfg(test)]
mod tests {
    use super::*;

    // comprehensive tests are handled in the C++ Qt unit test framework
    // there are a couple here to make iterating within this file faster

    #[test]
    fn serialize_and_deserialize() {
        let dna = holochain_dna_create();
        let dna_json_raw = holochain_dna_to_json(dna);
        holochain_dna_free(dna);

        let dna2 = holochain_dna_create_from_json(dna_json_raw);
        holochain_dna_string_free(dna_json_raw);

        let version_raw = holochain_dna_get_dna_spec_version(dna2);
        let version_str = unsafe { CStr::from_ptr(version_raw).to_string_lossy().into_owned() };
        assert_eq!(version_str, String::from("2.0"));
        holochain_dna_string_free(version_raw);

        holochain_dna_free(dna2);
    }

    #[test]
    fn can_set_and_get_value() {
        let val = CString::new("test").unwrap();

        let dna = holochain_dna_create();

        holochain_dna_set_name(dna, val.as_ptr());

        let res_raw = holochain_dna_get_name(dna);
        let res_str = unsafe { CStr::from_ptr(res_raw).to_string_lossy().into_owned() };

        assert_eq!(val.to_string_lossy(), res_str);

        holochain_dna_string_free(res_raw);
        holochain_dna_free(dna);
    }
}
