use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use hash_table::links_entry::*;
use json::ToJson;
use nucleus::ribosome::api::{
    HcApiReturnCode, Runtime,
};
use serde_json;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// ZomeApiFunction::LinkEntries function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: LinkEntriesArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_link_entries(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let input: Link = match serde_json::from_str(&args_str) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            // Return Error code in i32 format
            return Ok(Some(RuntimeValue::I32(HcApiReturnCode::ErrorJson as i32)));
        }
    };

    // Create AddLink Action
    let action_wrapper = ActionWrapper::new(Action::AddLink(input));
    // Send Action and block for result
    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        &runtime.action_channel,
        &runtime.observer_channel,
        action_wrapper.clone(),
        move |state: &::state::State| {
            let mut actions_copy = state.agent().actions();
            match actions_copy.remove(&action_wrapper) {
                Some(v) => {
                    // @TODO never panic in wasm
                    // @see https://github.com/holochain/holochain-rust/issues/159
                    sender
                        .send(v)
                        // the channel stays connected until the first message has been sent
                        // if this fails that means that it was called after having returned done=true
                        .expect("observer called after done");

                    true
                }
                None => false,
            }
        },
    );
    // TODO #97 - Return error if timeout or something failed
    // return Err(_);

    let action_result = receiver.recv().expect("observer dropped before done");

    match action_result {
        ActionResponse::LinkEntries(_) => {
            // serialize, allocate and encode result
            let json = action_result.to_json();
            match json {
                Ok(j) => runtime.store_utf8(&j),
                Err(_) => Ok(Some(RuntimeValue::I32(HcApiReturnCode::ErrorJson as i32))),
            }
        }
        _ => Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ErrorActionResult as i32,
        ))),
    }
}
