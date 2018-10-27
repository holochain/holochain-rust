use holochain_core_types::{entry_type::EntryType, hash::HashString};
use holochain_wasm_utils::api_serialization::ZomeApiGlobals;
use multihash::Hash as Multihash;
use nucleus::ribosome::Runtime;
use serde_json;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// ZomeApiFunction::InitGlobals secret function code
/// args: [0] encoded MemoryAllocation as u32
/// Not expecting any complex input
/// Returns an HcApiReturnCode as I32
pub fn invoke_init_globals(
    runtime: &mut Runtime,
    _args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // Create the ZomeApiGlobals struct with some default values
    let mut globals = ZomeApiGlobals {
        dna_name: runtime.dna_name.to_string(),
        dna_hash: HashString::from(""),
        agent_id_str: runtime.context.agent.to_string(),
        // TODO #233 - Implement agent pub key hash
        agent_address: HashString::encode_from_str("FIXME-agent_address", Multihash::SHA2256),
        agent_initial_hash: HashString::from(""),
        agent_latest_hash: HashString::from(""),
    };
    // Update fields
    if let Some(state) = runtime.context.state() {
        // Update dna_hash
        if let Some(dna) = state.nucleus().dna() {
            globals.dna_hash =
                HashString::encode_from_serializable(dna.to_json(), Multihash::SHA2256);
        }
        // Update agent hashes
        let maybe_top = state.agent().top_chain_header();
        if maybe_top.is_some() {
            let mut found_entries: Vec<HashString> = vec![];
            for chain_header in state
                .agent()
                .chain()
                .iter_type(&maybe_top, &EntryType::AgentId)
            {
                found_entries.push(chain_header.entry_address().to_owned());
            }
            if found_entries.len() > 0 {
                globals.agent_latest_hash = found_entries[0].clone();
                globals.agent_initial_hash = found_entries.pop().unwrap();
                globals.agent_address = globals.agent_latest_hash.clone();
            }
        }
    };
    // Store it in wasm memory
    return runtime.store_utf8(&serde_json::to_string(&globals).unwrap());
}

#[cfg(test)]
pub mod tests {
    use holochain_agent::Agent;
    use holochain_core_types::cas::content::AddressableContent;
    use holochain_wasm_utils::api_serialization::ZomeApiGlobals;
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };

    #[test]
    /// test that bytes passed to debug end up in the log
    fn test_init_globals() {
        let input: Vec<u8> = vec![];
        let (mut call_result, _) =
            test_zome_api_function(ZomeApiFunction::InitGlobals.as_str(), input);
        call_result.pop(); // Remove trailing character
        let globals: ZomeApiGlobals = serde_json::from_str(&call_result).unwrap();
        assert_eq!(globals.dna_name, "TestApp");
        // TODO #233 - Implement agent address
        // assert_eq!(obj.agent_address, "QmScgMGDzP3d9kmePsXP7ZQ2MXis38BNRpCZBJEBveqLjD");
        assert_eq!(globals.agent_id_str, "jane");
        assert_eq!(
            globals.agent_initial_hash,
            Agent::from("jane".to_string()).address()
        );
        assert_eq!(globals.agent_initial_hash, globals.agent_latest_hash);
    }
}
