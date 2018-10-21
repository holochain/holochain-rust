extern crate directories;
extern crate holochain_agent;
extern crate holochain_cas_implementations;
extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_core_types;
extern crate holochain_dna;

use holochain_cas_implementations::{
    cas::file::FilesystemStorage,
    eav::file::EavFileStorage,
    path::{create_path_if_not_exists, storage_path},
};
use holochain_core::context::Context;
use holochain_core_api::Holochain;
use holochain_core_types::error::HolochainError;
use holochain_dna::Dna;
use std::sync::Arc;

use directories::UserDirs;

use holochain_agent::Agent;
use holochain_core::{logger::Logger, persister::SimplePersister};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    sync::Mutex,
};

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

#[no_mangle]
pub unsafe extern "C" fn holochain_new(ptr: *mut Dna) -> *mut Holochain {
    let context = get_context();

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

fn get_context() -> Result<Context, HolochainError> {
    let agent = Agent::from("c_bob".to_string());
    match UserDirs::new() {
        Some(user_dir) => {
            let home_dir = user_dir.home_dir();
            let cas_path = storage_path(home_dir, "cas")?;
            let eav_path = storage_path(home_dir, "eav")?;
            create_path_if_not_exists(&cas_path)?;
            create_path_if_not_exists(&eav_path)?;
            Context::new(
                agent,
                Arc::new(Mutex::new(NullLogger {})),
                Arc::new(Mutex::new(SimplePersister::new())),
                FilesystemStorage::new(&cas_path).unwrap(),
                EavFileStorage::new(eav_path).unwrap(),
            )
        }
        None => Err(HolochainError::ErrorGeneric(
            "Could not create context".to_string(),
        )),
    }
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
        Ok(string_result) => {
            let string_trim = string_result.trim_right_matches(char::from(0));
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
