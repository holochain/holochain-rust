//! This crate is an ffi wrapper to provide a c-compatible dna library.
//!
//! Remember to free all dna objects and returned strings.
//!
//! See the associated Qt unit tests in the c_binding_tests directory.
#![feature(try_from)]
extern crate holochain_core_types;

use holochain_core_types::dna::Dna;
use std::{
    convert::TryFrom,
    ffi::{CStr, CString},
    os::raw::c_char,
    panic::catch_unwind,
};

use holochain_core_types::json::JsonString;

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub extern "C" fn holochain_dna_create() -> *mut Dna {
    match catch_unwind(|| Box::into_raw(Box::new(Dna::new()))) {
        Ok(r) => r,
        #[cfg_attr(tarpaulin, skip)]
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub extern "C" fn holochain_dna_create_from_json(buf: *const c_char) -> *mut Dna {
    match catch_unwind(|| {
        let json = unsafe { CStr::from_ptr(buf).to_string_lossy().into_owned() };

        let dna = Dna::try_from(JsonString::from(json)).expect("could not restore DNA from JSON");

        Box::into_raw(Box::new(dna))
    }) {
        Ok(r) => r,
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub extern "C" fn holochain_dna_free(ptr: *mut Dna) {
    catch_unwind(|| {
        if ptr.is_null() {
            return;
        }
        unsafe {
            Box::from_raw(ptr);
        }
    })
    .unwrap_or(());
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub extern "C" fn holochain_dna_to_json(ptr: *const Dna) -> *mut c_char {
    match catch_unwind(|| {
        let dna = unsafe {
            assert!(!ptr.is_null());
            &*ptr
        };

        let json_string = JsonString::from(dna.to_owned());

        let json_cstring = match CString::new(String::from(json_string)) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        json_cstring.into_raw()
    }) {
        Ok(r) => r,
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub extern "C" fn holochain_dna_string_free(s: *mut c_char) {
    catch_unwind(|| {
        if s.is_null() {
            return;
        }
        unsafe {
            CString::from_raw(s);
        }
    })
    .unwrap_or(());
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
            })
            .unwrap_or(());
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

#[repr(C)]
pub struct CStringVec {
    len: usize,
    ptr: *const *const c_char,
}

unsafe fn vec_char_to_cstringvec(vec: Option<Vec<*const c_char>>, string_vec: *mut CStringVec) {
    match vec {
        Some(function_names) => {
            (*string_vec).len = function_names.len();
            (*string_vec).ptr = function_names.as_ptr();
            std::mem::forget(function_names);
        }
        None => {
            (*string_vec).len = 0;
            (*string_vec).ptr = std::ptr::null_mut();
        }
    }
}

fn zome_names_as_vec(dna: &Dna) -> Option<Vec<*const c_char>> {
    Some(
        dna.zomes
            .keys()
            .map(|zome_name| {
                let raw = match CString::new(zome_name.to_string()) {
                    Ok(s) => s.into_raw(),
                    Err(_) => std::ptr::null(),
                };
                raw as *const c_char
            })
            .collect::<Vec<*const c_char>>(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn holochain_dna_get_zome_names(ptr: *mut Dna, string_vec: *mut CStringVec) {
    let dna = &*ptr;
    let zome_names = zome_names_as_vec(dna);
    vec_char_to_cstringvec(zome_names, string_vec);
}

unsafe fn cstring_vec_to_rustvec(string_vec: *mut CStringVec) -> Vec<CString> {
    let vec = Vec::from_raw_parts(
        (*string_vec).ptr as *mut *const c_char,
        (*string_vec).len,
        (*string_vec).len,
    );
    vec.into_iter()
        .map(|s| CString::from_raw(s as *mut c_char))
        .collect::<Vec<_>>()
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub unsafe extern "C" fn holochain_dna_free_zome_names(string_vec: *mut CStringVec) {
    let _vec = cstring_vec_to_rustvec(string_vec);
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
fn capabilities_as_vec(dna: &Dna, zome_name: &str) -> Option<Vec<*const c_char>> {
    let result = dna
        .zomes
        .get(zome_name)?
        .capabilities
        .keys()
        .map(|cap_name| {
            let raw = match CString::new(cap_name.clone()) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null(),
            };
            raw as *const c_char
        })
        .collect::<Vec<*const c_char>>();
    Some(result)
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub unsafe extern "C" fn holochain_dna_get_capabilities_names(
    ptr: *mut Dna,
    zome_name: *const c_char,
    string_vec: *mut CStringVec,
) {
    let dna = &*ptr;
    let zome_name = CStr::from_ptr(zome_name).to_string_lossy();
    let capabalities = capabilities_as_vec(dna, &*zome_name);
    vec_char_to_cstringvec(capabalities, string_vec);
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
fn fn_names_as_vec(dna: &Dna, zome_name: &str) -> Option<Vec<*const c_char>> {
    let result = dna
        .zomes
        .get(zome_name)?
        .fn_declarations
        .iter()
        .map(|fn_declaration| {
            let raw = match CString::new(fn_declaration.name.clone()) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null(),
            };
            raw as *const c_char
        })
        .collect::<Vec<*const c_char>>();
    Some(result)
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub unsafe extern "C" fn holochain_dna_get_function_names(
    ptr: *mut Dna,
    zome_name: *const c_char,
    string_vec: *mut CStringVec,
) {
    let dna = &*ptr;

    let zome_name = CStr::from_ptr(zome_name).to_string_lossy();

    let fn_names = fn_names_as_vec(dna, &*zome_name);
    vec_char_to_cstringvec(fn_names, string_vec)
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
fn fn_parameters_as_vec(
    dna: &Dna,
    zome_name: &str,
    function_name: &str,
) -> Option<Vec<*const c_char>> {
    let result = dna
        .get_function_with_zome_name(zome_name, function_name)
        .ok()?
        .inputs
        .iter()
        .map(|input| {
            let raw = match CString::new(input.name.clone()) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null(),
            };
            raw as *const c_char
        })
        .collect::<Vec<*const c_char>>();
    Some(result)
}

#[cfg_attr(tarpaulin, skip)] //Tested in c_bindings_test by C based test code
#[no_mangle]
pub unsafe extern "C" fn holochain_dna_get_function_parameters(
    ptr: *mut Dna,
    zome_name: *const c_char,
    function_name: *const c_char,
    string_vec: *mut CStringVec,
) {
    let dna = &*ptr;

    let zome_name = CStr::from_ptr(zome_name).to_string_lossy();
    let function_name = CStr::from_ptr(function_name).to_string_lossy();

    let fn_parameters = fn_parameters_as_vec(dna, &*zome_name, &*function_name);
    vec_char_to_cstringvec(fn_parameters, string_vec)
}

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

    #[test]
    fn test_holochain_dna_get_zome_names() {
        let mut dna = Dna::try_from(JsonString::from(
            r#"{
                "name": "test",
                "description": "test",
                "version": "test",
                "uuid": "00000000-0000-0000-0000-000000000000",
                "dna_spec_version": "2.0",
                "properties": {
                    "test": "test"
                },
                "zomes": {
                    "test zome": {
                        "name": "test zome",
                        "description": "test",
                        "config": {},
                        "capabilities": {
                            "test capability": {
                                "type": "public",
                                "fn_declarations": [],
                                "code": {
                                    "code": ""
                                }
                            }
                        },
                        "entry_types": {}
                    },
                    "test zome2": {
                        "name": "test zome",
                        "description": "test",
                        "config": {},
                        "capabilities": {
                            "test capability": {
                                "type": "public",
                                "fn_declarations": [],
                                "code": {
                                    "code": ""
                                }
                            }
                        },
                        "entry_types": {}
                    }
                }
            }"#,
        ))
        .unwrap();

        let mut cnames = CStringVec {
            len: 0,
            ptr: 0 as *const *const c_char,
        };
        unsafe { holochain_dna_get_zome_names(&mut dna, &mut cnames) };

        assert_eq!(cnames.len, 2);

        let names = unsafe { cstring_vec_to_rustvec(&mut cnames) };
        let names = names
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>();

        assert!(names[0] == "test zome" || names[1] == "test zome");
        assert!(names[0] == "test zome2" || names[1] == "test zome2");
        assert!(names[0] != names[1]);
    }
}
