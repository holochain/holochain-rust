extern crate hdk;

use hdk::prelude::*;

fn handle_call_bridge() -> JsonString {
    hdk::call("test-callee", "greeter", Address::from("token"), "hello", JsonString::empty_object()).unwrap()
}

fn handle_call_bridge_error() -> JsonString {
    hdk::call("test-callee", "greeter", Address::from("token"), "non-existent-function", JsonString::empty_object()).into()
}

define_zome! {
    entries: []

    init: || {
        CallbackResult::Pass
    }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        ValidationResult::Ok
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
