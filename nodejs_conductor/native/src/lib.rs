#![feature(try_from)]
#![feature(await_macro)]
#[macro_use]
extern crate neon;
extern crate neon_serde;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate colored;
extern crate holochain_cas_implementations;
extern crate holochain_conductor_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_net;
extern crate holochain_node_test_waiter;
extern crate tempfile;

mod config;
pub mod js_test_conductor;

use crate::{config::js_make_config, js_test_conductor::JsTestConductor};

register_module!(mut m, {
    m.export_function("makeConfig", js_make_config)?;
    m.export_class::<JsTestConductor>("TestConductor")?;
    Ok(())
});
