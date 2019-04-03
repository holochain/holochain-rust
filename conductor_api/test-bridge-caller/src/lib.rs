#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use hdk::holochain_core_types::{
    error::HolochainError,
    json::JsonString,
    cas::content::Address,
};

fn handle_call_bridge() -> JsonString {
    hdk::call("test-callee", "greeter", Address::from("token"), "hello", JsonString::empty_object()).unwrap()
}

define_zome! {
    entries: []

    genesis: || {
        Ok(())
    }

    functions: [
        call_bridge: {
            inputs: | |,
            outputs: |result: JsonString|,
            handler: handle_call_bridge
        }
    ]

    traits: {
        hc_public [call_bridge]
    }
}
