#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use hdk::holochain_core_types::error::HolochainError;
use hdk::holochain_core_types::json::JsonString;

fn handle_call_bridge() -> JsonString {
    hdk::call("test-callee", "greeter", "token", "hello", JsonString::from("{}")).unwrap()
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

    capabilities: {
        public (Public) [call_bridge]
    }
}
