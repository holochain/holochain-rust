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

fn handle_call_bridge_error() -> JsonString {
    hdk::call("test-callee", "greeter", Address::from("token"), "non-existent-function", JsonString::empty_object()).into()
}

define_zome! {
    entries: []

    init: || {
        Ok(())
    }

    functions: [
        call_bridge: {
            inputs: | |,
            outputs: |result: JsonString|,
            handler: handle_call_bridge
        }

        call_bridge_error: {
            inputs: | |,
            outputs: |result: JsonString|,
            handler: handle_call_bridge_error
        }
    ]

    traits: {
        hc_public [call_bridge, call_bridge_error]
    }
}
