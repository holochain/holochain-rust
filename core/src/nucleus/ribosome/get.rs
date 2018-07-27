use agent::ActionResult;
use nucleus::ribosome::{HcApiReturnCode, Runtime};
use serde_json;
use snowflake;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

#[derive(Deserialize, Default, Debug, Serialize)]
struct GetArgs {
    key: String,
}

pub fn invoke_get(runtime: &mut Runtime, args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
    // @TODO assert or return error?
    // @see https://github.com/holochain/holochain-rust/issues/159
    assert!(args.len() == 2);

    // assert!(false);

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
    let res_entry: Result<GetArgs, _> = serde_json::from_str(&arg);
    // Exit on error
    if res_entry.is_err() {
        // Return Error code in i32 format
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ERROR_SERDE_JSON as i32,
        )));
    }

    let input = res_entry.unwrap();

    let action = ::agent::Action::Get {
        key: input.key.clone(),
        id: snowflake::ProcessUniqueId::new(),
    };

    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        &runtime.action_channel,
        &runtime.observer_channel,
        ::state::Action::Agent(action.clone()),
        move |state: &::state::State| {
            let actions = state.agent().actions().clone();
            if actions.contains_key(&action) {
                // @TODO is this unwrap OK since we check the key exists above?
                let v = actions.get(&action).unwrap();
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
            // params.push(0); // Add string terminate character (important)

            assert!(0 == mem_offset);

            // TODO #65 - use our Malloc instead
            runtime
                .memory
                .set(mem_offset, &params)
                .expect("memory should be writable");

            // Return success in i32 format
            Ok(Some(RuntimeValue::I32(HcApiReturnCode::SUCCESS as i32)))
        }
        _ => {
            panic!("action result of get not get of result action");
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test_utils;
    extern crate wabt;

    use holochain_dna::zome::capabilities::ReservedCapabilityNames;
    use nucleus::ribosome::call;
    // use instance::Observer;
    use self::wabt::Wat2Wasm;
    // use std::sync::mpsc::channel;
    use super::GetArgs;
    use hash_table::entry::tests::test_entry;
    use serde_json;
    use tests::create_instance;
    // use test_utils;

    pub fn test_args_bytes() -> Vec<u8> {
        let args = GetArgs {
            key: test_entry().hash().into(),
        };
        serde_json::to_string(&args).unwrap().into_bytes()
    }

    pub fn test_wasm() -> Vec<u8> {
        let wasm_binary = Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(
                // We don't expect everyone to be a pro at hand-coding WASM so...
                //
                // How this works:
                //
                // only need 1 page of memory for testing
                // (memory 1)
                //
                // all modules compiled with rustc must have an export named "memory" (or fatal)
                // (export "memory" (memory 0))
                //
                // define and export the *_dispatch function that will be called from the
                // ribosome rust implementation, where * is the fourth arg to `call`
                // @see nucleus::ribosome::call
                // (func (export "*_dispatch") ...)
                //
                // define the memory offset and length that the serialized input struct can be
                // found across as params to the exported function, also the function return type
                // (param $offset i32)
                // (param $length i32)
                // (result i32)
                r#"
(module
    (import "env" "get" (func $get (type 0)))

    (memory 1)
    (export "memory" (memory 0))

    (type (;0;) (func (param i32) (param i32) (result i32)))

    (func (export "test_get_dispatch")
        (param $offset i32)
        (param $length i32)
        (result i32)

        (call
            $get
            (get_local $offset)
            (get_local $length)
        )
    )
)
                "#,
            )
            .unwrap();

        wasm_binary.as_ref().to_vec()
    }

    #[test]
    fn test_get_round_trip() {
        let dna = test_utils::create_test_dna_with_wasm(
            "test_zome__get".into(),
            ReservedCapabilityNames::LifeCycle.as_str().to_string(),
            test_wasm(),
        );
        let instance = create_instance(dna);

        // let (action_channel, _action_receive) = channel::<::state::ActionWrapper>();
        // let (tx_observer, _observer) = channel::<Observer>();
        let runtime = call(
            &instance.action_channel(),
            &instance.observer_channel(),
            test_wasm(),
            "test_get",
            Some(test_args_bytes()),
        ).expect("test_get should be callable");

        // @TODO
        let b = runtime.memory.get(0, 222).unwrap();
        let s = String::from_utf8(b).unwrap();
        assert_eq!(
            "{\"header\":{\"entry_type\":\"testEntryType\",\"time\":\"\",\"next\":null,\"entry\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}",
            s,
        );
    }

}
