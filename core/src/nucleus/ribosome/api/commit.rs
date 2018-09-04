use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use json::ToJson;
use nucleus::ribosome::{
    api::{runtime_allocate_encode_str, runtime_args_to_utf8, HcApiReturnCode, Runtime},
    callback::{validate_commit::validate_commit, CallbackParams, CallbackResult},
};
use serde_json;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// Struct for input data received when Commit API function is invoked
#[derive(Deserialize, Default, Debug, Serialize)]
struct CommitArgs {
    entry_type_name: String,
    entry_content: String,
}

/// HcApiFuncIndex::COMMIT function code
/// args: [0] encoded MemoryAllocation as u32
/// expected complex argument: r#"{"entry_type_name":"post","entry_content":"hello"}"#
/// Returns an HcApiReturnCode as I32
pub fn invoke_commit_entry(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime_args_to_utf8(&runtime, &args);
    let entry_input: CommitArgs = match serde_json::from_str(&args_str) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            // Return Error code in i32 format
            return Ok(Some(RuntimeValue::I32(HcApiReturnCode::ErrorJson as i32)));
        }
    };

    // Create Chain Entry
    let entry =
        ::hash_table::entry::Entry::new(&entry_input.entry_type_name, &entry_input.entry_content);

    // @TODO test that failing validation prevents commits happening
    // @see https://github.com/holochain/holochain-rust/issues/206
    if let CallbackResult::Fail(_) = validate_commit(
        &runtime.action_channel,
        &runtime.observer_channel,
        &runtime.function_call.zome,
        &CallbackParams::ValidateCommit(entry.clone()),
    ) {
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ErrorCallbackResult as i32,
        )));
    }
    // anything other than a fail means we should commit the entry

    // Create Commit Action
    let action_wrapper = ActionWrapper::new(Action::Commit(entry));
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
        ActionResponse::Commit(_) => {
            // serialize, allocate and encode result
            let json = action_result.to_json();
            match json {
                Ok(j) => runtime_allocate_encode_str(runtime, &j),
                Err(_) => Ok(Some(RuntimeValue::I32(HcApiReturnCode::ErrorJson as i32))),
            }
        }
        _ => Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ErrorActionResult as i32,
        ))),
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use super::CommitArgs;
    use hash_table::entry::tests::test_entry;
    use key::Key;
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function_runtime, ZomeAPIFunction},
        Defn,
    };
    use serde_json;

    /// dummy commit args from standard test entry
    pub fn test_commit_args_bytes() -> Vec<u8> {
        let e = test_entry();
        let args = CommitArgs {
            entry_type_name: e.entry_type().into(),
            entry_content: e.content().into(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    #[test]
    /// test that we can round trip bytes through a commit action and get the result from WASM
    fn test_commit_round_trip() {
        let (runtime, _) = test_zome_api_function_runtime(
            ZomeAPIFunction::CommitEntry.as_str(),
            test_commit_args_bytes(),
        );

        assert_eq!(
            runtime.result,
            format!(r#"{{"hash":"{}"}}"#, test_entry().key()) + "\u{0}",
        );
    }

}
