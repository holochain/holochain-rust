use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use holochain_core_types::{error::HcResult, json::JsonString, signature::Signature};
use holochain_dpki::keypair::generate_random_sign_keypair;
use holochain_sodium::secbuf::SecBuf;
use holochain_wasm_utils::api_serialization::sign::{OneTimeSignArgs, SignArgs, SignOneTimeResult};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Sign function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_sign(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let sign_args = match SignArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            context.log(format!(
                "err/zome: invoke_sign failed to deserialize SignArgs: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let signature = context
        .sign(sign_args.payload.clone())
        .map(|sig| JsonString::from_json(&sig));

    context.log(format!(
        "debug/zome: signature of data:{:?} by:{:?} is:{:?}",
        sign_args.payload, context.agent_id, signature
    ));

    runtime.store_result(signature)
}

/// ZomeApiFunction::SignOneTime function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_sign_one_time(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let sign_args = match OneTimeSignArgs::try_from(args_str.clone()) {
        Ok(sign_input) => sign_input,
        // Exit on error
        Err(err) => {
            context.log(format!(
                "err/zome: invoke_sign_one_time failed to deserialize OneTimeSignArgs: {:?} got err: {:?}",
                args_str, err
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    runtime.store_result(sign_one_time(sign_args.payloads))
}

/// creates a one-time private key and sign data returning the signature and the public key
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
    use crate::nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::{cas::content::Address, json::JsonString};
    use holochain_dpki::utils::verify;

    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_function_sign() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::Sign.as_str(),
            r#"{ "payload": "this is data" }"#.as_bytes().to_vec(),
        );
        assert_eq!(JsonString::from_json(r#"{"ok":true,"value":"xoEEoLF1yWM4VBNtjEwrfM/iVzjuAxxbkOyBWi0LV0+1CAH/PCs9MErnbmFeZRtQNtw7+SmVrm7Irac4lZsaDA==","error":"null"}"#), call_result,);
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
