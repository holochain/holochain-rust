use super::runtime_args_to_utf8;
use agent::ActionResult;
use nucleus::ribosome::{HcApiReturnCode, Runtime};
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
/// args: [0] memory offset where complex argument is stored
/// args: [1] memory length of complex argument soted in memory
/// expected complex argument: r#"{"entry_type_name":"post","entry_content":"hello"}"#
/// Returns an HcApiReturnCode as I32
pub fn invoke_commit(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime_args_to_utf8(&runtime, &args);
    let res_entry: Result<CommitArgs, _> = serde_json::from_str(&args_str);
    // Exit on error
    if res_entry.is_err() {
        // Return Error code in i32 format
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ERROR_SERDE_JSON as i32,
        )));
    }

    // Create Chain Entry
    let entry_input = res_entry.unwrap();
    let entry =
        ::hash_table::entry::Entry::new(&entry_input.entry_type_name, &entry_input.entry_content);

    // Create Commit Action
    let action = ::agent::Action::commit(&entry);

    // Send Action and block for result
    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        &runtime.action_channel,
        &runtime.observer_channel,
        ::state::Action::Agent(action.clone()),
        move |state: &::state::State| {
            let actions = state.agent().actions().clone();
            if actions.contains_key(&action) {
                // @TODO is this unwrap OK since we check the key exists above?
                let v = &actions[&action];
                sender.send(v.clone()).expect("local channel to be open");
                true
            } else {
                false
            }
        },
    );
    // TODO #97 - Return error if timeout or something failed
    // return Err(_);

    let action_result = receiver.recv().expect("local channel to work");

    match action_result {
        ActionResult::Commit(hash) => {
            // write JSON pair to memory
            let params_str = format!("{{\"hash\":\"{}\"}}", hash);
            let mut params: Vec<_> = params_str.into_bytes();
            params.push(0); // Add string terminate character (important)

            // TODO #65 - use our Malloc instead
            runtime
                .memory
                .set(args.nth(0), &params)
                .expect("memory should be writable");

            // Return success in i32 format
            Ok(Some(RuntimeValue::I32(HcApiReturnCode::SUCCESS as i32)))
        }
        _ => Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ERROR_ACTION_RESULT as i32,
        ))),
    }
}

/// HcApiFuncIndex::COMMIT function code
/// args: [0] encoded MemoryAllocation as u32
/// expected complex argument: r#"{"entry_type_name":"post","entry_content":"hello"}"#
/// Returns an HcApiReturnCode as I32
fn invoke_commit(runtime: &mut Runtime, args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
    assert!(args.len() == 1);

    // Read complex argument serialized in memory
    let encoded_allocation: u32 = args.nth(0);
    let allocation = SinglePageAllocation::new(encoded_allocation);
    let allocation = allocation.expect("received error instead of valid encoded allocation");
    let bin_arg = runtime.memory_manager.read(allocation);

    // deserialize complex argument
    let arg = String::from_utf8(bin_arg).unwrap();
    let res_entry: Result<CommitInputStruct, _> = serde_json::from_str(&arg);
    // Exit on error
    if res_entry.is_err() {
        // Return Error code in i32 format
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ERROR_SERDE_JSON as i32,
        )));
    }

    // Create Chain Entry
    let entry_input = res_entry.unwrap();
    let entry =
        ::hash_table::entry::Entry::new(&entry_input.entry_type_name, &entry_input.entry_content);

    // Create Commit Action
    let action_commit = ::state::Action::Agent(::agent::Action::Commit(entry.clone()));

    // Send Action and block for result
    // TODO #97 - Dispatch with observer so we can check if the action did its job without errors
    ::instance::dispatch_action_and_wait(
        &runtime.action_channel,
        &runtime.observer_channel,
        action_commit.clone(),
        // TODO #131 - add timeout argument and return error on timeout
        // REDUX_DEFAULT_TIMEOUT_MS,
    );
    // TODO #97 - Return error if timeout or something failed
    // return Err(_);

    // Hash entry
    let hash_str = entry.hash();

    // Write Hash of Entry in memory in output format
    let result_str = format!("{{\"hash\":\"{}\"}}", hash_str);
    let mut result: Vec<_> = result_str.into_bytes();
    result.push(0); // Add string terminate character (important)

    let allocation_of_result = runtime.memory_manager.write(&result);
    if allocation_of_result.is_err() {
        return Err(Trap::new(TrapKind::MemoryAccessOutOfBounds));
    }

    // Return encoded allocation of result
    let encoded_allocation = allocation_of_result.unwrap().encode();
    Ok(Some(RuntimeValue::I32(encoded_allocation as i32)))
}

#[cfg(test)]
mod tests {
    extern crate test_utils;
    extern crate wabt;

    use super::CommitArgs;
    use nucleus::ribosome::tests::test_zome_api_function_runtime;
    use hash_table::entry::tests::test_entry;
    use serde_json;

    pub fn test_args_bytes() -> Vec<u8> {
        let e = test_entry();
        let args = CommitArgs {
            entry_type_name: e.entry_type().into(),
            entry_content: e.content().into(),
        };
        serde_json::to_string(&args).unwrap().into_bytes()
    }

    #[test]
    fn test_get_round_trip() {
        let runtime = test_zome_api_function_runtime("commit", test_args_bytes());

        // @TODO
        let b = runtime.memory.get(0, 58).unwrap();
        let s = String::from_utf8(b).unwrap();
        assert_eq!(
            format!(r#"{{"hash":"{}"}}"#, test_entry().key()) + "\u{0}",
            s,
        );
    }

}
