use crate::{
    context::Context,
    nucleus::CallbackFnCall,
    wasm_engine::{
        self,
        callback::{Callback, CallbackParams, CallbackResult},
        runtime::WasmCallData,
        Defn,
    },
};

use holochain_json_api::{error::JsonError, json::JsonString};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct ReceiveReturnValue(Result<String, String>);

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn receive(
    context: Arc<Context>,
    zome: &str,
    // we ignore params for init
    parameters: &CallbackParams,
) -> CallbackResult {
    let params = match parameters {
        CallbackParams::Receive(params) => params,
        _ => return CallbackResult::NotImplemented("receive/1".into()),
    };

    let call = CallbackFnCall::new(
        zome,
        &Callback::Receive.as_str().to_string(),
        JsonString::from(params),
    );

    match wasm_engine::run_dna(
        Some(call.clone().parameters.to_bytes()),
        WasmCallData::new_callback_call(context, call),
    ) {
        Ok(call_result) => CallbackResult::ReceiveResult(call_result.to_string()),
        Err(err) => CallbackResult::Fail(err.to_string()),
    }
}

#[cfg(test)]
pub mod tests {

    use super::receive;
    use crate::{
        instance::tests::test_context,
        wasm_engine::{
            callback::{tests::test_callback_instance, Callback, CallbackParams, CallbackResult},
            Defn,
        },
    };
    use holochain_persistence_api::cas::content::Address;
    use holochain_wasm_utils::api_serialization::receive::ReceiveParams;

    #[test]
    fn receive_fail() {
        let zome = "test_zome";
        let netname = Some("receive_fail test");
        let instance = test_callback_instance(
            zome,
            // anything other than Init is fine here
            Callback::MissingNo.as_str(),
            0,
            netname,
        )
        .expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test", netname));

        if let CallbackResult::Fail(_) = receive(
            context,
            zome,
            &CallbackParams::Receive(ReceiveParams {
                from: Address::from(""),
                payload: String::from(""),
            }),
        ) {
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

        let result = receive(
            context,
            zome,
            &CallbackParams::Receive(ReceiveParams {
                from: Address::from(""),
                payload: String::from(""),
            }),
        );

        assert_eq!(CallbackResult::ReceiveResult(String::from("null")), result);
    }
}
