use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use hash::HashString;
use json::ToJson;
use nucleus::ribosome::api::{HcApiReturnCode, Runtime};
use serde_json;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

#[derive(Deserialize, Default, Debug, Serialize)]
struct GetAppEntryArgs {
    key: HashString,
}

/// ZomeApiFunction::GetAppEntry function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: GetEntryArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_get_entry(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let res_entry: Result<GetAppEntryArgs, _> = serde_json::from_str(&args_str);
    // Exit on error
    if res_entry.is_err() {
        // Return Error code in i32 format
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ArgumentDeserializationFailed as i32,
        )));
    }
    let input = res_entry.unwrap();

    let action_wrapper = ActionWrapper::new(Action::GetEntry(input.key));

    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        &runtime.context.action_channel,
        &runtime.context.observer_channel,
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
        ActionResponse::GetEntry(maybe_entry) => {
            // serialize, allocate and encode result
            let json_str = maybe_entry.expect("should be valid json entry").to_json();
            match json_str {
                Ok(json) => runtime.store_utf8(&json),
                Err(_) => Ok(Some(RuntimeValue::I32(
                    HcApiReturnCode::ResponseSerializationFailed as i32,
                ))),
            }
        }
        _ => Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ReceivedWrongActionResult as i32,
        ))),
    }
}

#[cfg(test)]
mod tests {
    extern crate test_utils;
    extern crate wabt;

    use self::wabt::Wat2Wasm;
    use super::GetAppEntryArgs;
    use chain::SourceChain;
    use hash_table::entry::tests::test_entry;
    use instance::tests::{test_context_and_logger, test_instance};
    use key::Key;
    use nucleus::{
        ribosome::api::{
            call,
            commit::tests::test_commit_args_bytes,
            tests::{test_capability, test_parameters, test_zome_name},
        },
        ZomeFnCall,
    };
    use serde_json;
    use std::sync::Arc;

    /// dummy get args from standard test entry
    pub fn test_get_args_bytes() -> Vec<u8> {
        let args = GetAppEntryArgs {
            key: test_entry().hash().into(),
        };
        serde_json::to_string(&args).unwrap().into_bytes()
    }

    /// wat string that exports both get and a commit dispatches so we can test a round trip
    pub fn test_get_round_trip_wat() -> Vec<u8> {
        Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(
                r#"
(module
    (import "env" "hc_get_entry"
        (func $get
            (param i32)
            (result i32)
        )
    )

    (import "env" "hc_commit_entry"
        (func $commit
            (param i32)
            (result i32)
        )
    )

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "get_dispatch")
            (param $allocation i32)
            (result i32)

        (call
            $get
            (get_local $allocation)
        )
    )

    (func
        (export "commit_dispatch")
            (param $allocation i32)
            (result i32)

        (call
            $commit
            (get_local $allocation)
        )
    )

    (func
        (export "validate_commit")
        (param $allocation i32)
        (result i32)

        (i32.const 0)
    )
)
                "#,
            )
            .unwrap()
            .as_ref()
            .to_vec()
    }

    #[test]
    /// test that we can round trip bytes through a get action and it comes back from wasm
    fn test_get_round_trip() {
        let wasm = test_get_round_trip_wat();
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability(),
            wasm.clone(),
        );
        let instance = test_instance(dna.clone());
        let (context, _) = test_context_and_logger("joan");
        let context = instance.initialize_context(context);

        println!("{:?}", instance.state().agent().chain().top_pair());
        println!(
            "{:?}",
            instance
                .state()
                .agent()
                .chain()
                .top_pair()
                .expect("could not get top pair")
                .expect("top pair was None")
                .key()
        );

        let commit_call = ZomeFnCall::new(
            &test_zome_name(),
            &test_capability(),
            "commit_dispatch",
            &test_parameters(),
        );
        let commit_runtime = call(
            &dna.name.to_string(),
            Arc::clone(&context),
            wasm.clone(),
            &commit_call,
            Some(test_commit_args_bytes()),
        ).expect("test should be callable");

        assert_eq!(
            commit_runtime.result,
            format!(r#"{{"hash":"{}"}}"#, test_entry().key()) + "\u{0}",
        );

        let get_call = ZomeFnCall::new(
            &test_zome_name(),
            &test_capability(),
            "get_dispatch",
            &test_parameters(),
        );
        let get_runtime = call(
            &dna.name.to_string(),
            Arc::clone(&context),
            wasm.clone(),
            &get_call,
            Some(test_get_args_bytes()),
        ).expect("test should be callable");

        let mut expected = "".to_owned();
        expected
            .push_str("{\"value\":\"test entry content\",\"entry_type\":\"testEntryType\"}\u{0}");

        assert_eq!(expected, get_runtime.result);
    }

}
