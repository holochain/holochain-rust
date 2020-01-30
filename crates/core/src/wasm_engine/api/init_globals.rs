use crate::{wasm_engine::{api::ZomeApiResult, Runtime},NEW_RELIC_LICENSE_KEY};
use holochain_core_types::entry::entry_type::EntryType;

use holochain_persistence_api::{
    cas::content::{Address, AddressableContent},
    hash::HashString,
};

use holochain_json_api::json::JsonString;

use holochain_wasm_utils::api_serialization::ZomeApiGlobals;
use wasmi::RuntimeArgs;

/// ZomeApiFunction::InitGlobals secret function code
/// args: [0] encoded MemoryAllocation as u64
/// Not expecting any complex input
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_init_globals(runtime: &mut Runtime, _args: &RuntimeArgs) -> ZomeApiResult {
    let call_data = runtime.call_data()?;
    let dna = runtime
        .context()?
        .get_dna()
        .expect("No DNA found in invoke_init_globals");
    let dna_name = dna.name.clone();
    // Create the ZomeApiGlobals struct with some default values
    let mut globals = ZomeApiGlobals {
        dna_name,
        dna_address: Address::from(""),
        agent_id_str: JsonString::from(call_data.context.agent_id.clone()).to_string(),
        agent_address: call_data.context.agent_id.address(),
        agent_initial_hash: HashString::from(""),
        agent_latest_hash: HashString::from(""),
        public_token: Address::from(""),
        cap_request: runtime
            .zome_call_data()
            .map(|zome_call_data| Some(zome_call_data.call.cap))
            .unwrap_or_else(|_| None),
        properties: JsonString::from(dna.properties),
    };

    // Update fields
    if let Some(state) = call_data.context.state() {
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
                .chain_store()
                .iter_type(&maybe_top, &EntryType::AgentId)
            {
                found_entries.push(chain_header.entry_address().to_owned());
            }
            if !found_entries.is_empty() {
                globals.agent_latest_hash = found_entries[0].clone();
                globals.agent_initial_hash = found_entries.pop().unwrap();
                globals.agent_address = globals.agent_latest_hash.clone();
            }
        }
    };

    // Update public_token
    let maybe_token = call_data.context.get_public_token();
    if let Ok(token) = maybe_token {
        globals.public_token = token;
    }

    // Store it in wasm memory
    runtime.store_result(Ok(globals))
}

#[cfg(test)]
pub mod tests {
    use crate::wasm_engine::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::{
        dna::capabilities::CapabilityRequest, error::ZomeApiInternalResult, signature::Signature,
    };
    use holochain_json_api::json::JsonString;
    use holochain_persistence_api::cas::content::Address;
    use holochain_wasm_utils::api_serialization::ZomeApiGlobals;
    use std::convert::TryFrom;
    use test_utils::mock_signing::registered_test_agent;

    #[test]
    /// test that the correct globals values are created for zome calls
    fn test_init_globals() {
        let input: Vec<u8> = vec![];
        let (call_result, _) = test_zome_api_function(ZomeApiFunction::InitGlobals.as_str(), input);

        let zome_api_internal_result = ZomeApiInternalResult::try_from(call_result).unwrap();
        let globals =
            ZomeApiGlobals::try_from(JsonString::from_json(&zome_api_internal_result.value))
                .unwrap();

        assert_eq!(globals.dna_name, "TestApp");
        let expected_agent = registered_test_agent("jane");
        assert_eq!(
            globals.agent_address.to_string(),
            expected_agent.pub_sign_key
        );
        // TODO (david.b) this should work:
        //assert_eq!(globals.agent_id_str, String::from(AgentId::generate_fake("jane")));
        // assert_eq!(
        //     globals.agent_initial_hash,
        //     AgentId::generate_fake("jane").address()
        // );
        assert_eq!(globals.agent_initial_hash, globals.agent_latest_hash);

        // this hash should stay the same as long as the public functions in the test zome
        // don't change.
        assert_eq!(
            globals.public_token,
            Address::from("QmdZiJWdVCh8s38tCcAAq8f7HpHkd9KLFnHh9vLTddt8D2"),
        );

        assert_eq!(
            globals.cap_request,
            Some(CapabilityRequest::new( Address::from("dummy_token"),
                                    Address::from("HcSCimiBHJ8y3zejkjtHsu9Q8MZx96ztvfYRJ9fJH3Pbxodac5s8rqmShYqaamz"),
                                    Signature::from("nI/AFdqZPYw1yoCeV92pKWwugdkB54JJDhLLf3JgMFl9sm3aFIWKpiRo+4t8L+wn+S0Pg1Vh0Bzbmq3DSfJwDw=="),
                                    )),
        );
    }
}
