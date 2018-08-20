
use nucleus::ribosome::api::{runtime_allocate_encode_str, Runtime};
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

use serde_json;

#[derive(Serialize)]
struct InitGlobalsOutput {
    app_name: String,
    app_dna_hash: String,
    app_key_hash: String,
    app_agent_hash: String,
    app_agent_top_hash: String,
    app_agent_str: String,
}

/// HcApiFuncIndex::INIT_GLOBALS secret function code
/// args: [0] encoded MemoryAllocation as u32
/// Not expecting any complex input
/// Returns an HcApiReturnCode as I32
pub fn invoke_init_globals(
    runtime: &mut Runtime,
    _args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {

    let globals = InitGlobalsOutput {
        app_name: "FIXME-app_name".to_string(),
        app_dna_hash: "FIXME-app_dna_hash".to_string(),
        app_key_hash: "FIXME-app_key_hash".to_string(),
        app_agent_hash: "FIXME-app_agent_hash".to_string(),
        app_agent_top_hash: "FIXME-app_agent_top_hash".to_string(),
        app_agent_str: "FIXME-app_agent_str".to_string(),
    };

    return runtime_allocate_encode_str(
        runtime,
        &serde_json::to_string(&globals).unwrap());
}


#[cfg(test)]
pub mod tests {
    //use nucleus::ribosome::api::tests::test_zome_api_function_runtime;

    #[test]
    /// test that bytes passed to debug end up in the log
    fn test_init_globals() {
//        let (_runtime, logger) = test_zome_api_function_runtime("debug", test_args_bytes());
//        let result = logger.lock();
//        match result {
//            Err(_) => assert!(false),
//            Ok(logger) => {
//                assert_eq!(format!("{:?}", logger.log), "[\"foo\"]".to_string());
//            }
//        }
    }
}
