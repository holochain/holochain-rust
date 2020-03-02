use holochain_json_api::json::*;
use holochain_wasm_types::crypto::CryptoArgs;
use crate::workflows::WorkflowResult;
use crate::context::Context;
use std::sync::Arc;

/// ZomeApiFunction::Sign function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn crypto_workflow(context: Arc<Context>, crypto_args: &CryptoArgs) -> WorkflowResult<JsonString> {
    let message = context
        .conductor_api
        .execute(crypto_args.payload.clone(), crypto_args.method.clone())
        .map(|sig| JsonString::from_json(&sig));

    log_debug!(
        context,
        "zome: crypto method {:?} of data:{:?} by:{:?} is:{:?}",
        crypto_args.method,
        crypto_args.payload,
        context.agent_id,
        message
    );

    message
}

#[cfg(test)]
mod test_super {
    use crate::wasm_engine::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::error::ZomeApiInternalResult;
    use holochain_json_api::json::*;
    use std::convert::TryFrom;
    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_crypto_functions() {
        let (call_result_json, _) = test_zome_api_function(
            ZomeApiFunction::Crypto.as_str(),
            r#"{ "payload": "this is data", "method" : "Encrypt" }"#
                .as_bytes()
                .to_vec(),
        );

        let encrypt_result = ZomeApiInternalResult::try_from(call_result_json)
            .expect("Could not try from zomeapiitnernal");
        assert!(encrypt_result.ok);

        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::Crypto.as_str(),
            format!(
                r#"{{ "payload": "{}", "method" : "Decrypt" }}"#,
                encrypt_result.value
            )
            .as_bytes()
            .to_vec(),
        );
        assert_eq!(
            JsonString::from_json(r#"{"ok":true,"value":"this is data","error":"null"}"#),
            call_result,
        );
    }

    #[test]
    fn test_zome_api_crypto_signing() {
        let payload = r#"{ "payload": "test ' payload", "method" : "Sign" }"#;
        let (call_result_json, _) = test_zome_api_function(
            ZomeApiFunction::Crypto.as_str(),
            payload.as_bytes().to_vec(),
        );
        println!("Crypto::Sign( {:?} ) == {:?}", payload, call_result_json);
        assert_eq!(
            JsonString::from_json(
                r#"{"ok":true,"value":"ZDwPQ2TX9Xiq1k73JWczzqWr97rmdAodWWInlGfFjKiE0wFgMc2WvhmaFpNfrCv3y5uSOOLD5MgJqAeDsKb4Cw==","error":"null"}"#
            ),
            call_result_json
        );

        let payload = r#"{ "payload": "test \" payload", "method" : "Sign" }"#;
        let (call_result_json, _) = test_zome_api_function(
            ZomeApiFunction::Crypto.as_str(),
            payload.as_bytes().to_vec(),
        );
        println!("Crypto::Sign( {:?} ) == {:?}", payload, call_result_json);
        assert_eq!(
            JsonString::from_json(
                r#"{"ok":true,"value":"ODn3OE9jcZPfB403T7lFJbySVU4Ugu2Kv/kpkg50lD1cJ5E+gDs3zWwADJjQzkps+qp03k6C5ygegcGd2ERoCA==","error":"null"}"#
            ),
            call_result_json
        );
    }
}
