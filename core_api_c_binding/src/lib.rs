extern crate directories;
extern crate holochain_cas_implementations;
extern crate holochain_container_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_net;

use holochain_container_api::{context_builder::ContextBuilder, Holochain};
use holochain_core::context::Context;
use holochain_core_types::{cas::content::Address, dna::Dna, error::HolochainError};

use std::sync::Arc;

use holochain_core::logger::Logger;
use holochain_core_types::{
    agent::AgentId,
    dna::capabilities::{CallSignature, CapabilityCall},
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

#[no_mangle]
pub unsafe extern "C" fn holochain_new(ptr: *mut Dna, storage_path: CStrPtr) -> *mut Holochain {
    let path = CStr::from_ptr(storage_path).to_string_lossy().into_owned();
    let context = get_context(&path);

    assert!(!ptr.is_null());
    let dna = Box::from_raw(ptr);

    match context {
        Ok(con) => match Holochain::new(*dna, Arc::new(con)) {
            Ok(hc) => Box::into_raw(Box::new(hc)),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn holochain_load(storage_path: CStrPtr) -> *mut Holochain {
    let path = CStr::from_ptr(storage_path).to_string_lossy().into_owned();
    let context = get_context(&path);

    match context {
        Ok(con) => match Holochain::load(path, Arc::new(con)) {
            Ok(hc) => Box::into_raw(Box::new(hc)),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

fn get_context(path: &String) -> Result<Context, HolochainError> {
    let agent = AgentId::generate_fake("c_bob");
    Ok(ContextBuilder::new()
        .with_agent(agent)
        .with_file_storage(path.clone())?
        .spawn())
}

#[no_mangle]
pub unsafe extern "C" fn holochain_start(ptr: *mut Holochain) -> bool {
    let holochain = {
        if ptr.is_null() {
            return false;
        }
        &mut *ptr
    };

    holochain.start().is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn holochain_stop(ptr: *mut Holochain) -> bool {
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
pub unsafe extern "C" fn holochain_call(
    ptr: *mut Holochain,
    zome: CStrPtr,
    token: CStrPtr,
    function: CStrPtr,
    parameters: CStrPtr,
) -> CStrPtr {
    if ptr.is_null() || zome.is_null() || function.is_null() || parameters.is_null() {
        return std::ptr::null_mut();
    }

    let holochain = &mut *ptr;
    let zome = CStr::from_ptr(zome).to_string_lossy().into_owned();
    let token = CStr::from_ptr(token).to_string_lossy().into_owned();
    let function = CStr::from_ptr(function).to_string_lossy().into_owned();
    let parameters = CStr::from_ptr(parameters).to_string_lossy().into_owned();

    match holochain.call(
        zome.as_str(),
        CapabilityCall::new(
            Address::from(token.as_str()),
            Address::from("fake_caller"),
            CallSignature::default(),
        ), // FIXME: caller
        function.as_str(),
        parameters.as_str(),
    ) {
        Ok(json_string_result) => {
            let string_result = String::from(json_string_result);
            let string_trim = string_result.trim_end_matches(char::from(0));
            match CString::new(string_trim) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(holochain_error) => match CString::new(format!(
            "Error calling zome function: {:?}",
            holochain_error
        )) {
            Ok(s) => s.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
    }
}
