use holochain_core_types::{error::HcResult, signature::Signature};
use holochain_dpki::keypair::generate_random_sign_keypair;
use holochain_wasm_types::sign::{OneTimeSignArgs, SignOneTimeResult};
use lib3h_sodium::secbuf::SecBuf;
use crate::wasm_engine::runtime::Runtime;

/// ZomeApiFunction::SignOneTime function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_sign_one_time(_: &mut Runtime, sign_args: OneTimeSignArgs) -> HcResult<SignOneTimeResult> {
    sign_one_time(sign_args.payloads)
}

/// creates a one-time private key and sign data returning the signature and the public key
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn sign_one_time(payloads: Vec<String>) -> HcResult<SignOneTimeResult> {
    let mut sign_keys = generate_random_sign_keypair()?;
    let mut signatures = Vec::new();
    for data in payloads {
        let mut data_buf = SecBuf::with_insecure_from_string(data);

        let mut signature_buf = sign_keys.sign(&mut data_buf)?;
        let buf = signature_buf.read_lock();
        // Return as base64 encoded string
        let signature_str = base64::encode(&**buf);
        signatures.push(Signature::from(signature_str))
    }
    Ok(SignOneTimeResult {
        pub_key: sign_keys.public,
        signatures,
    })
}

#[cfg(test)]
mod test_super {
    use super::sign_one_time;
    use crate::wasm_engine::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_dpki::utils::verify;
    use holochain_json_api::json::JsonString;
    use holochain_persistence_api::cas::content::Address;

    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_function_sign() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::Crypto.as_str(),
            r#"{ "payload": "this is data", "method" : "Sign" }"#
                .as_bytes()
                .to_vec(),
        );
        assert_eq!(
            JsonString::from_json(
                r#"{"ok":true,"value":"xoEEoLF1yWM4VBNtjEwrfM/iVzjuAxxbkOyBWi0LV0+1CAH/PCs9MErnbmFeZRtQNtw7+SmVrm7Irac4lZsaDA==","error":"null"}"#
            ),
            call_result,
        );
    }

    #[test]
    fn test_sign_one_time() {
        let data = base64::encode("the data to sign");
        let more_data = base64::encode("more data to sign");
        let result = sign_one_time(vec![data.clone(), more_data.clone()]);
        assert!(!result.is_err());

        let result = result.unwrap();

        assert_eq!(result.signatures.len(), 2);

        let sig1 = result.signatures[0].clone();
        let sig2 = result.signatures[1].clone();

        let source = Address::from(result.pub_key);
        let vresult = verify(source.clone(), data, sig1.clone());
        assert!(!vresult.is_err());
        assert!(vresult.unwrap());

        let vresult = verify(source, more_data, sig2.clone());
        assert!(!vresult.is_err());
        assert!(vresult.unwrap());

        assert_ne!(sig1, sig2);
    }
}
