use crate::{
    context::Context,
    nucleus::{
        ribosome::{
            self,
            callback::{Callback, CallbackParams, CallbackResult},
            runtime::WasmCallData,
            Defn,
        },
        CallbackFnCall,
    },
};
use holochain_core_types::{error::HolochainError, json::JsonString};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct ReceiveReturnValue(Result<String, String>);

pub fn receive(
    context: Arc<Context>,
    zome: &str,
    // we ignore params for genesis
    parameters: &CallbackParams,
) -> CallbackResult {
    let params = match parameters {
        CallbackParams::Receive(payload) => payload,
        _ => return CallbackResult::NotImplemented("receive/1".into()),
    };

    let call = CallbackFnCall::new(
        zome,
        &Callback::Receive.as_str().to_string(),
        JsonString::from_json(&params),
    );

    let dna = context.get_dna().expect("Callback called without DNA set!");

    let maybe_wasm = dna.get_wasm_from_zome_name(zome);
    if maybe_wasm.is_none() {
        return CallbackResult::NotImplemented("receive/2".into());
    }
    let wasm = maybe_wasm.unwrap();
    if wasm.code.is_empty() {
        return CallbackResult::NotImplemented("receive/3".into());
    }

    match ribosome::run_dna(
        wasm.code.clone(),
        Some(call.clone().parameters.to_bytes()),
        WasmCallData::new_callback_call(context, dna.name, call),
    ) {
        Ok(call_result) => CallbackResult::ReceiveResult(call_result.to_string()),
        Err(_) => CallbackResult::NotImplemented("receive/4".into()),
    }
}

#[cfg(test)]
pub mod tests {

    use super::receive;
    use crate::{
        instance::tests::test_context,
        nucleus::ribosome::{
            callback::{tests::test_callback_instance, Callback, CallbackParams, CallbackResult},
            Defn,
        },
    };

    #[test]
    fn not_implemented() {
        let zome = "test_zome";
        let netname = Some("not_implemented test");
        let instance = test_callback_instance(
            zome,
            // anything other than Genesis is fine here
            Callback::MissingNo.as_str(),
            0,
            netname,
        )
        .expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test", netname));

        if let CallbackResult::NotImplemented(_) =
            receive(context, zome, &CallbackParams::Receive(String::from("")))
        {
            ()
        } else {
            panic!("unexpected result");
        }
    }

    #[test]
    fn implemented_with_null() {
        let zome = "test_zome";
        let netname = Some("implemented_with_null");
        let instance = test_callback_instance(zome, Callback::Receive.as_str(), 0, netname)
            .expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test", netname));

        let result = receive(context, zome, &CallbackParams::Receive(String::from("")));

        assert_eq!(CallbackResult::ReceiveResult(String::from("null")), result);
    }
}
