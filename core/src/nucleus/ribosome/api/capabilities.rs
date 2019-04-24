use crate::{
    agent::actions::commit::commit_entry,
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
};
use holochain_core_types::{
    cas::content::Address,
    entry::{cap_entries::CapTokenGrant, Entry},
    error::HolochainError,
};
use holochain_wasm_utils::api_serialization::capabilities::GrantCapabilityArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

pub fn invoke_grant_capability(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let args = match GrantCapabilityArgs::try_from(args_str.clone()) {
        Ok(input) => input,
        Err(..) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    let task_result: Result<Address, HolochainError> =
        match CapTokenGrant::create(&args.id, args.cap_type, args.assignees, args.functions) {
            Ok(grant) => context.block_on(commit_entry(
                Entry::CapTokenGrant(grant.clone()),
                None,
                &context.clone(),
            )),
            Err(err) => Err(HolochainError::ErrorGeneric(format!(
                "Unable to commit capability grant: {}",
                err
            ))),
        };

    runtime.store_result(task_result)
}

#[cfg(test)]
pub mod tests {
    use crate::nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::{
        cas::content::Address, entry::cap_entries::CapabilityType, error::ZomeApiInternalResult,
        json::JsonString,
    };
    use holochain_wasm_utils::api_serialization::capabilities::GrantCapabilityArgs;
    use std::collections::BTreeMap;

    /// dummy args
    pub fn test_grant_capability_args_bytes() -> Vec<u8> {
        let mut functions = BTreeMap::new();
        functions.insert("test_zome".to_string(), vec!["test_function".to_string()]);
        let grant_args = GrantCapabilityArgs {
            id: "some_id".to_string(),
            cap_type: CapabilityType::Assigned,
            assignees: Some(vec![Address::from("fake address")]),
            functions: functions,
        };

        JsonString::from(grant_args).to_bytes()
    }

    #[test]
    /// test that we can round trip bytes through a grant_capability action and get the result from WASM
    fn test_grant_capability_round_trip() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::GrantCapability.as_str(),
            test_grant_capability_args_bytes(),
        );

        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(JsonString::from(ZomeApiInternalResult::success(
                    Address::from("Qma8KWBHZwiXNBJ4PBtT4uDUVgPAyUJASHumThZMTPAAJe")
                ))) + "\u{0}")
            ),
        );
    }
}
