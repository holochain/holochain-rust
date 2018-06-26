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
use std::sync::Mutex;

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

#[no_mangle]
pub extern "C" fn hc_new(ptr: *mut Dna) -> *mut Holochain {
    let agent = Agent::from_string("c_bob");

    let context = Arc::new(Context {
        agent,
        logger: Arc::new(Mutex::new(NullLogger {})),
        persister: Arc::new(Mutex::new(SimplePersister::new())),
    });

    let dna = unsafe {
        assert!(!ptr.is_null());
        Box::from_raw(ptr)
    };

    match Holochain::new(*dna, context) {
        Ok(hc) => Box::into_raw(Box::new(hc)),
        Err(_) => std::ptr::null_mut(),
    }
}
