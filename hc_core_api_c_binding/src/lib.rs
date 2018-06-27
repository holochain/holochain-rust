#![deny(warnings)]
extern crate hc_agent;
extern crate hc_core;
extern crate hc_core_api;
extern crate hc_dna;

use hc_core::context::Context;
use hc_core_api::Holochain;
use hc_dna::Dna;
use std::sync::Arc;

use hc_agent::Agent;
use hc_core::logger::Logger;
use hc_core::persister::SimplePersister;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

#[no_mangle]
pub unsafe extern "C" fn hc_new(ptr: *mut Dna) -> *mut Holochain {
    let agent = Agent::from_string("c_bob");

    let context = Arc::new(Context {
        agent,
        logger: Arc::new(Mutex::new(NullLogger {})),
        persister: Arc::new(Mutex::new(SimplePersister::new())),
    });

    assert!(!ptr.is_null());
    let dna = Box::from_raw(ptr);

    match Holochain::new(*dna, context) {
        Ok(hc) => Box::into_raw(Box::new(hc)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn hc_start(ptr: *mut Holochain) -> bool {
    let holochain = {
        if ptr.is_null() {
            return false;
        }
        &mut *ptr
    };

    holochain.start().is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn hc_stop(ptr: *mut Holochain) -> bool {
    let holochain = {
        if ptr.is_null() {
            return false;
        }
        &mut *ptr
    };

    holochain.stop().is_ok()
}

type CStrPtr = *mut c_char;

#[no_mangle]
pub unsafe extern "C" fn hc_call(
    ptr: *mut Holochain,
    zome: CStrPtr,
    capability: CStrPtr,
    function: CStrPtr,
    parameters: CStrPtr,
) -> CStrPtr {
    if ptr.is_null()
        || zome.is_null()
        || capability.is_null()
        || function.is_null()
        || parameters.is_null()
    {
        return std::ptr::null_mut();
    }

    let holochain = &mut *ptr;
    let zome = CStr::from_ptr(zome).to_string_lossy().into_owned();
    let capability = CStr::from_ptr(capability).to_string_lossy().into_owned();
    let function = CStr::from_ptr(function).to_string_lossy().into_owned();
    let parameters = CStr::from_ptr(parameters).to_string_lossy().into_owned();

    match holochain.call(
        zome.as_str(),
        capability.as_str(),
        function.as_str(),
        parameters.as_str(),
    ) {
        Ok(string_result) => match CString::new(string_result) {
            Ok(s) => s.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(holochain_error) => match CString::new(format!(
            "Error calling zome function: {:?}",
            holochain_error
        )) {
            Ok(s) => s.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
    }
}
