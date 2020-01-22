use crate::{
    signal::{Signal, UserSignal},
    wasm_engine::{api::ZomeApiResult, Runtime},
};
use holochain_wasm_utils::api_serialization::emit_signal::EmitSignalArgs;
use wasmer_runtime::Value;

/// ZomeApiFunction::EmitSignal function code
/// args: [0] encoded MemoryAllocation as u64
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I64
pub fn invoke_emit_signal(runtime: &Runtime, emit_signal_args: EmitSignalArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    if let Some(sender) = context.signal_tx() {
        let signal = Signal::User(UserSignal::from(emit_signal_args));
        let _ = sender.send(signal).map_err(|err| {
            log_error!(
                context,
                "zome: invoke_emit_signal() could not send signal: {:?}",
                err,
            );
        });
    } else {
        log_error!(context, "zome: invoke_emit_signal() could not send signal because signal channel is not set up!");
    }

    // We only log this case but still return Ok(()) since the semantic of sending a signal
    // is all about decoupling sender and receiver - if nobody is listening, the sender
    // should not care..
    ribosome_success!()
}

#[cfg(test)]
pub mod tests {
    use crate::{
        instance::tests::test_instance_and_context,
        signal::{Signal, UserSignal},
        wasm_engine::{
            api::{
                tests::{test_zome_api_function_call, test_zome_api_function_wasm, test_zome_name},
                ZomeApiFunction,
            },
            Defn,
        },
    };
    use crossbeam_channel::unbounded;
    use holochain_json_api::json::JsonString;
    use holochain_wasm_utils::api_serialization::emit_signal::EmitSignalArgs;
    use std::sync::Arc;

    pub fn test_signal() -> UserSignal {
        UserSignal::from(test_args())
    }

    pub fn test_args() -> EmitSignalArgs {
        EmitSignalArgs {
            name: String::from("test-signal"),
            arguments: JsonString::from_json("{message: \"Hello\"}"),
        }
    }

    pub fn test_args_bytes() -> Vec<u8> {
        let args_string: JsonString = test_args().into();
        args_string.to_string().into_bytes()
    }

    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_function_emit_signal() {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::EmitSignal.as_str());
        let dna = test_utils::create_test_dna_with_wasm(&test_zome_name(), wasm.clone());

        let (_instance, context) =
            test_instance_and_context(dna, None).expect("Could not create test instance");

        let (tx, rx) = unbounded::<Signal>();
        let mut context = (*context).clone();
        context.signal_tx = Some(tx);
        let context = Arc::new(context);

        let args_string: JsonString = test_args().into();
        println!("{}", args_string.to_string());

        let _ = test_zome_api_function_call(context.clone(), test_args_bytes());

        let received = rx.try_recv();
        assert!(received.is_ok());
        let signal = received.unwrap();
        if let Signal::User(user_signal) = signal {
            assert_eq!(user_signal, test_signal());
        } else {
            assert!(false, "Expected a Signal::User");
        }
    }
}
