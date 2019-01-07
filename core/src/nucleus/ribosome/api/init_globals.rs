use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::entry_type::EntryType,
    hash::HashString,
    json::JsonString,
};
use holochain_wasm_utils::api_serialization::ZomeApiGlobals;
use multihash::Hash as Multihash;
use wasmi::RuntimeArgs;

/// ZomeApiFunction::InitGlobals secret function code
/// args: [0] encoded MemoryAllocation as u32
/// Not expecting any complex input
/// Returns an HcApiReturnCode as I32
pub fn invoke_init_globals(runtime: &mut Runtime, _args: &RuntimeArgs) -> ZomeApiResult {
    // Create the ZomeApiGlobals struct with some default values
    let mut globals = ZomeApiGlobals {
        dna_name: runtime.dna_name.to_string(),
        dna_address: Address::from(""),
        agent_id_str: JsonString::from(runtime.context.agent_id.clone()).to_string(),
        // TODO #233 - Implement agent pub key hash
        agent_address: Address::from(""),
        //agent_address: Address::encode_from_str("FIXME-agent_address", Multihash::SHA2256),
        agent_initial_hash: HashString::from(""),
        agent_latest_hash: HashString::from(""),
    };

    // Update fields
    if let Some(state) = runtime.context.state() {
        // Update dna_address
        if let Some(dna) = state.nucleus().dna() {
            globals.dna_address = dna.address()
        }
        // Update agent hashes
        let maybe_top = state.agent().top_chain_header();
        if maybe_top.is_some() {
            let mut found_entries: Vec<Address> = vec![];
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
                // TODO #233 - Implement agent pub key hash
                globals.agent_address = globals.agent_latest_hash.clone();
            }
        }
    };

    // Store it in wasm memory
    runtime.store_result(Ok(globals))
}

#[cfg(test)]
pub mod tests {
    use crate::nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::{agent::AgentId, error::ZomeApiInternalResult, json::JsonString};
    use holochain_wasm_utils::api_serialization::ZomeApiGlobals;
    use std::convert::TryFrom;

    #[test]
    /// test that bytes passed to debug end up in the log
    fn test_init_globals() {
        let input: Vec<u8> = vec![];
        let (call_result, _) = test_zome_api_function(ZomeApiFunction::InitGlobals.as_str(), input);

        let zome_api_internal_result = ZomeApiInternalResult::try_from(call_result).unwrap();
        let globals =
            ZomeApiGlobals::try_from(JsonString::from(zome_api_internal_result.value)).unwrap();

        assert_eq!(globals.dna_name, "TestApp");
        // TODO #233 - Implement agent address
        let expected_agent = AgentId::generate_fake("jane");
        assert_eq!(globals.agent_address.to_string(), expected_agent.key);
        // TODO (david.b) this should work:
        //assert_eq!(globals.agent_id_str, String::from(AgentId::generate_fake("jane")));
        // assert_eq!(
        //     globals.agent_initial_hash,
        //     AgentId::generate_fake("jane").address()
        // );
        assert_eq!(globals.agent_initial_hash, globals.agent_latest_hash);
    }
}
