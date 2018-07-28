use super::runtime_args_to_utf8;
use agent::ActionResult;
use nucleus::ribosome::{HcApiReturnCode, Runtime};
use serde_json;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

#[derive(Deserialize, Default, Debug, Serialize)]
struct GetArgs {
    key: String,
}

pub fn invoke_get(runtime: &mut Runtime, args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime_args_to_utf8(&runtime, &args);
    let res_entry: Result<GetArgs, _> = serde_json::from_str(&args_str);
    // Exit on error
    if res_entry.is_err() {
        // Return Error code in i32 format
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ERROR_SERDE_JSON as i32,
        )));
    }

    let input = res_entry.unwrap();

    let action = ::agent::Action::get(&input.key);

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
        ActionResult::Get(maybe_pair) => {
            let pair_str = maybe_pair.and_then(|p| Some(p.json())).unwrap_or_default();

            // write JSON pair to memory
            let mut params: Vec<_> = pair_str.to_string().into_bytes();
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

#[cfg(test)]
mod tests {
    extern crate test_utils;
    extern crate wabt;

    use super::GetArgs;
    use hash_table::entry::tests::test_entry;
    use serde_json;
    use nucleus::ribosome::tests::test_zome_api_function_runtime;

    pub fn test_args_bytes() -> Vec<u8> {
        let args = GetArgs {
            key: test_entry().hash().into(),
        };
        serde_json::to_string(&args).unwrap().into_bytes()
    }

    #[test]
    fn test_get_round_trip() {
        let runtime = test_zome_api_function_runtime("get", test_args_bytes());

        // @TODO
        let b = runtime.memory.get(0, 223).unwrap();
        let s = String::from_utf8(b).unwrap();
        assert_eq!(
            "{\"header\":{\"entry_type\":\"testEntryType\",\"time\":\"\",\"next\":null,\"entry\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}\u{0}",
            s,
        );
    }

}
