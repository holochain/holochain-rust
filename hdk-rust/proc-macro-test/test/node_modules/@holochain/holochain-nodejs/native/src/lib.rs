#![feature(try_from)]
#![feature(await_macro)]
#![warn(unused_extern_crates)]
#[macro_use]
extern crate neon;
#[macro_use]
extern crate serde_derive;
extern crate holochain_conductor_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_node_test_waiter;

mod config;
pub mod js_test_conductor;

use crate::{config::js_make_config, js_test_conductor::JsTestConductor};

register_module!(mut m, {
    m.export_function("makeConfig", js_make_config)?;
    m.export_class::<JsTestConductor>("TestConductor")?;
    Ok(())
});
