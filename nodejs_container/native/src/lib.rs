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
extern crate holochain_container_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_net;
extern crate tempfile;

mod config;
pub mod js_test_container;
mod waiter;

use crate::{
    config::{js_instance_id, js_make_config},
    js_test_container::JsTestContainer,
};

register_module!(mut m, {
    m.export_function("makeConfig", js_make_config)?;
    m.export_function("makeInstanceId", js_instance_id)?;
    m.export_class::<JsTestContainer>("TestContainer")?;
    Ok(())
});
