extern crate hdk;

use hdk::prelude::*;

fn handle_call_bridge() -> ZomeApiResult<RawString> {
    Ok(hdk::call("test-callee", "greeter", Address::from("token"), "hello", JsonString::empty_object())?)
}

fn handle_call_bridge_error() -> ZomeApiResult<RawString> {
    Ok(hdk::call("test-callee", "greeter", Address::from("token"), "non-existent-function", JsonString::empty_object())?)
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
            outputs: |result: ZomeApiResult<RawString>|,
            handler: handle_call_bridge
        }

        call_bridge_error: {
            inputs: | |,
            outputs: |result: ZomeApiResult<RawString>|,
            handler: handle_call_bridge_error
        }
    ]

    traits: {
        hc_public [call_bridge, call_bridge_error]
    }
}
