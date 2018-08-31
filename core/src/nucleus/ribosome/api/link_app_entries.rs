use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use nucleus::ribosome::{
    api::{runtime_allocate_encode_str, runtime_args_to_utf8, HcApiReturnCode, Runtime},
    // callback::{validate_commit::validate_commit, CallbackParams, CallbackResult},
};
use serde_json;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};
use hash_table::{
    // HashString, entry::Entry, sys_entry::ToEntry,
                 links_entry::*};

/// Struct for input data received when Commit API function is invoked
//#[derive(Deserialize, Default, Debug, Serialize)]
//pub struct LinkEntriesArgs {
//    base: HashString,
//    target: HashString,
//    tag: String,
//}

/// ZomeApiFunction::LinkAppEntries function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: LinkEntriesArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_link_app_entries(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime_args_to_utf8(&runtime, &args);
    let input: Link = match serde_json::from_str(&args_str) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            // Return Error code in i32 format
            return Ok(Some(RuntimeValue::I32(
                HcApiReturnCode::ErrorSerdeJson as i32,
            )));
        }
    };

    // Create Commit Action
    // FIXME should be a LinkAppEntries Action
    let action_wrapper = ActionWrapper::new(Action::LinkAppEntries(input));
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
        ActionResponse::CommitEntry(_) => {
            // serialize, allocate and encode result
            runtime_allocate_encode_str(runtime, &action_result.to_json())
        }
        _ => Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ErrorActionResult as i32,
        ))),
    }
}

#[cfg(test)]
mod tests {
    extern crate test_utils;
    extern crate wabt;

    use super::*;
    use hash_table::entry::tests::test_entry;
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function_runtime, ZomeApiFunction},
        Defn,
    };
    use serde_json;

    /// dummy commit args from standard test entry
    pub fn test_args_bytes() -> Vec<u8> {
        // let e = test_entry();
        let args = Link::new("0x42","0x13","toto");

        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    // FIXME
//    #[test]
//    fn test_link_entries_round_trip() {
//        let (runtime, _) = test_zome_api_function_runtime(
//            ZomeApiFunction::LinkAppEntries.as_str(),
//            test_args_bytes(),
//        );
//
//        assert_eq!(
//            runtime.result,
//            format!(r#"{{"hash":"{}"}}"#, test_entry().key()) + "\u{0}",
//        );
//    }

}
