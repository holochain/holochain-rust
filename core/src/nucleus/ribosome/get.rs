use serde_json;
use nucleus::ribosome::Runtime;
use nucleus::ribosome::HcApiReturnCode;
use wasmi::RuntimeArgs;
use wasmi::RuntimeValue;
use wasmi::Trap;

#[derive(Deserialize, Default, Debug)]
struct GetInputStruct {
    key: String,
}

pub fn invoke_get(runtime: &mut Runtime, args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
    // @TODO assert or return error?
    // @see https://github.com/holochain/holochain-rust/issues/159
    assert!(args.len() == 1);

    // Read complex argument serialized in memory
    // @TODO use our Malloced data instead
    // @see https://github.com/holochain/holochain-rust/issues/65
    let mem_offset: u32 = args.nth(0);
    let mem_len: u32 = args.nth(1);
    let bin_arg = runtime
        .memory
        .get(mem_offset, mem_len as usize)
        // @TODO panic here?
        // @see https://github.com/holochain/holochain-rust/issues/159
        .expect("Successfully retrive the arguments");

    // deserialize complex argument
    // @TODO panic here?
    // @see https://github.com/holochain/holochain-rust/issues/159
    let arg = String::from_utf8(bin_arg).unwrap();
    let res_entry: Result<GetInputStruct, _> = serde_json::from_str(&arg);
    // Exit on error
    if res_entry.is_err() {
        // Return Error code in i32 format
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ERROR_SERDE_JSON as i32,
        )));
    }

    let input = res_entry.unwrap();

    // create Get Action
    let action = ::state::Action::Agent(::agent::Action::Get(input.key.clone()));

    // // Send Action and block for result
    // ::instance::call_and_wait_for_result(
    //     &runtime.action_channel,
    //     &runtime.observer_channel,
    //     action.clone(),
    // );

    // // @TODO how to get pair back from dispatch?
    // let pair = runtime.action_channel;

    // Write Hash of Entry in memory in output format
    // let params_str = format!("{{\"hash\":\"{}\"}}", hash_str);
    let params_str = input.key;
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
