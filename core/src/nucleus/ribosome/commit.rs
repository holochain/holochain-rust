use nucleus::ribosome::{HcApiReturnCode, Runtime};
use serde_json;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// Struct for input data received when Commit API function is invoked
#[derive(Deserialize, Default, Debug)]
struct CommitInputStruct {
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
    // @TODO assert or return error?
    // @see https://github.com/holochain/holochain-rust/issues/159
    assert!(args.len() == 2);

    // Read complex argument serialized in memory
    // @TODO use our Malloced data instead
    // @see https://github.com/holochain/holochain-rust/issues/65
    let mem_offset: u32 = args.nth(0);
    let mem_len: u32 = args.nth(1);
    let bin_arg = runtime
        .memory
        .get(mem_offset, mem_len as usize)
        .expect("Successfully retrieve the arguments");

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
    // @TODO seems wrong to use the entry hash rather than pair key
    // @see https://github.com/holochain/holochain-rust/issues/160
    let hash_str = entry.hash();

    // Write Hash of Entry in memory in output format
    let params_str = format!("{{\"hash\":\"{}\"}}", hash_str);
    let mut params: Vec<_> = params_str.into_bytes();
    params.push(0); // Add string terminate character (important)

    // TODO #65 - use our Malloc instead
    runtime
        .memory
        .set(mem_offset, &params)
        .expect("memory should be writable");

    // Return success in i32 format
    Ok(Some(RuntimeValue::I32(HcApiReturnCode::SUCCESS as i32)))
}
