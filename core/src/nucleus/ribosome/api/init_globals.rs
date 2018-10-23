use nucleus::ribosome::Runtime;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};
use holochain_core_types::{
    hash::HashString,
    app_globals::AppGlobals,
};

use serde_json;

/// ZomeApiFunction::InitGlobals secret function code
/// args: [0] encoded MemoryAllocation as u32
/// Not expecting any complex input
/// Returns an HcApiReturnCode as I32
pub fn invoke_init_globals(
    runtime: &mut Runtime,
    _args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    let globals = AppGlobals {
        app_name: runtime.dna_name.to_string(),

        // TODO #232 - Implement Dna hash
        app_dna_hash: HashString::from("FIXME-app_dna_hash"),

        app_agent_id_str: runtime.context.agent.to_string(),

        // TODO #233 - Implement agent pub key hash
        app_agent_key_hash: HashString::from("FIXME-app_agent_key_hash"),

        // TODO #234 - Implement agent identity entry hashes
        app_agent_initial_hash: HashString::from("FIXME-app_agent_initial_hash"),
        app_agent_latest_hash: HashString::from("FIXME-app_agent_latest_hash"),
    };

    return runtime.store_utf8(&serde_json::to_string(&globals).unwrap());
}

#[cfg(test)]
pub mod tests {
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction}, Defn,
    };

    #[test]
    /// test that bytes passed to debug end up in the log
    fn test_init_globals() {
        let input: Vec<u8> = vec![];
        let (call_result, _) = test_zome_api_function(ZomeApiFunction::InitGlobals.as_str(), input);
        assert_eq!(
            call_result,
            "{\"app_name\":\"TestApp\",\"app_dna_hash\":\"FIXME-app_dna_hash\",\"app_agent_id_str\":\"joan\",\"app_agent_key_hash\":\"FIXME-app_agent_key_hash\",\"app_agent_initial_hash\":\"FIXME-app_agent_initial_hash\",\"app_agent_latest_hash\":\"FIXME-app_agent_latest_hash\"}\u{0}"
        .to_string());
    }
}
