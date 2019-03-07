use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use wasmi::RuntimeArgs;

/// ZomeApiFunction::Sign function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_sign(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = dbg!(runtime.load_json_string_from_args(&args));

    let signature = runtime.context()?.sign(args_str.into());

    runtime.store_result(signature)
}

#[cfg(test)]
mod test_super {
    use crate::nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::json::JsonString;

    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_function_sign() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::Sign.as_str(),
            "test".to_string().into_bytes(),
        );
        assert_eq!(JsonString::from("{\"ok\":true,\"value\":\"+StjDIBItBYSefv3sezv8A+n7eBhKimq8KSmLSXmqH3Lwu+TLsUUdbXiwtC+Hzlb1Yi1smbqE7wg7q2xIC6XAw==\",\"error\":\"null\"}"), call_result,);
    }
}
