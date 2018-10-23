use nucleus::ribosome::Runtime;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};
use serde_json;
use holochain_wasm_utils::api_serialization::ZomeApiGlobals;
use holochain_core_types::hash::HashString;

/// ZomeApiFunction::InitGlobals secret function code
/// args: [0] encoded MemoryAllocation as u32
/// Not expecting any complex input
/// Returns an HcApiReturnCode as I32
pub fn invoke_init_globals(
    runtime: &mut Runtime,
    _args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let globals = ZomeApiGlobals {
        dna_name:           runtime.dna_name.to_string(),
        // TODO #232 - Implement Dna hash
        dna_hash:           HashString::from("FIXME-dna_hash"),
        agent_id_str:       runtime.context.agent.to_string(),
        // TODO #233 - Implement agent pub key hash
        agent_key_hash:     HashString::from("FIXME-agent_key_hash"),
        // TODO #234 - Implement agent identity entry hashes
        agent_initial_hash: HashString::from("FIXME-agent_initial_hash"),
        agent_latest_hash:  HashString::from("FIXME-agent_latest_hash"),
    };
    return runtime.store_utf8(&serde_json::to_string(&globals).unwrap());
}

#[cfg(test)]
pub mod tests {
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };

    #[test]
    /// test that bytes passed to debug end up in the log
    fn test_init_globals() {
        let input: Vec<u8> = vec![];
        let (call_result, _) = test_zome_api_function(ZomeApiFunction::InitGlobals.as_str(), input);
        assert_eq!(
            call_result,
            "{\"dna_name\":\"TestApp\",\"dna_hash\":\"FIXME-dna_hash\",\"agent_id_str\":\"joan\",\"agent_key_hash\":\"FIXME-agent_key_hash\",\"agent_initial_hash\":\"FIXME-agent_initial_hash\",\"agent_latest_hash\":\"FIXME-agent_latest_hash\"}\u{0}"
        .to_string());
    }
}
