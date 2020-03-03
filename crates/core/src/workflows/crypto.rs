use holochain_wasm_types::crypto::CryptoArgs;
use crate::workflows::WorkflowResult;
use crate::context::Context;
use std::sync::Arc;
use holochain_wasm_types::wasm_string::WasmString;
use holochain_wasm_types::crypto::CryptoMethod;

/// ZomeApiFunction::Sign function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn crypto_workflow(context: Arc<Context>, crypto_args: &CryptoArgs) -> WorkflowResult<String> {
    println!("crypto_workflow: {:?}", crypto_args);
    let message = context
        .conductor_api
        .execute(crypto_args.payload.clone(), crypto_args.method.clone());
    println!("crypto_workflow message: {:?}", message);

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

pub async fn encrypt_workflow(context: Arc<Context>, payload: &WasmString) -> WorkflowResult<WasmString> {
    crypto_workflow(
        Arc::clone(&context),
        &CryptoArgs {
            payload: payload.to_string(),
            method: CryptoMethod::Encrypt,
        }
    ).await.map(|encrypted_string| WasmString::from(encrypted_string))
}

pub async fn decrypt_workflow(context: Arc<Context>, payload: &WasmString) -> WorkflowResult<WasmString> {
    crypto_workflow(
        Arc::clone(&context),
        &CryptoArgs {
            payload: payload.to_string(),
            method: CryptoMethod::Decrypt,
        }
    ).await.map(|decrypted_string| WasmString::from(decrypted_string))
}

#[cfg(test)]
mod test_super {

    // use holochain_core_types::error::ZomeApiInternalResult;
    // use holochain_json_api::json::*;
    // use std::convert::TryFrom;
    // / test that bytes passed to debug end up in the log
    // #[test]
    // fn test_zome_api_crypto_functions() {
    //     let (call_result_json, _) = test_zome_api_function(
    //         ZomeApiFunction::Crypto.as_str(),
    //         r#"{ "payload": "this is data", "method" : "Encrypt" }"#
    //             .as_bytes()
    //             .to_vec(),
    //     );
    //
    //     let encrypt_result = ZomeApiInternalResult::try_from(call_result_json)
    //         .expect("Could not try from zomeapiitnernal");
    //     assert!(encrypt_result.ok);
    //
    //     let (call_result, _) = test_zome_api_function(
    //         ZomeApiFunction::Crypto.as_str(),
    //         format!(
    //             r#"{{ "payload": "{}", "method" : "Decrypt" }}"#,
    //             encrypt_result.value
    //         )
    //         .as_bytes()
    //         .to_vec(),
    //     );
    //     assert_eq!(
    //         JsonString::from_json(r#"{"ok":true,"value":"this is data","error":"null"}"#),
    //         call_result,
    //     );
    // }

    // #[test]
    // fn test_zome_api_crypto_signing() {
    //     let payload = r#"{ "payload": "test ' payload", "method" : "Sign" }"#;
    //     let (call_result_json, _) = test_zome_api_function(
    //         ZomeApiFunction::Crypto.as_str(),
    //         payload.as_bytes().to_vec(),
    //     );
    //     println!("Crypto::Sign( {:?} ) == {:?}", payload, call_result_json);
    //     assert_eq!(
    //         JsonString::from_json(
    //             r#"{"ok":true,"value":"ZDwPQ2TX9Xiq1k73JWczzqWr97rmdAodWWInlGfFjKiE0wFgMc2WvhmaFpNfrCv3y5uSOOLD5MgJqAeDsKb4Cw==","error":"null"}"#
    //         ),
    //         call_result_json
    //     );
    //
    //     let payload = r#"{ "payload": "test \" payload", "method" : "Sign" }"#;
    //     let (call_result_json, _) = test_zome_api_function(
    //         ZomeApiFunction::Crypto.as_str(),
    //         payload.as_bytes().to_vec(),
    //     );
    //     println!("Crypto::Sign( {:?} ) == {:?}", payload, call_result_json);
    //     assert_eq!(
    //         JsonString::from_json(
    //             r#"{"ok":true,"value":"ODn3OE9jcZPfB403T7lFJbySVU4Ugu2Kv/kpkg50lD1cJ5E+gDs3zWwADJjQzkps+qp03k6C5ygegcGd2ERoCA==","error":"null"}"#
    //         ),
    //         call_result_json
    //     );
    // }
}
