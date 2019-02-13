use crate::nucleus::{
    actions::call_zome_function::call_zome_function,
    ribosome::{api::ZomeApiResult, Runtime},
    ZomeFnCall,
};
use holochain_core_types::{error::HolochainError, json::JsonString};
use holochain_wasm_utils::api_serialization::{ZomeFnCallArgs, THIS_INSTANCE};
use jsonrpc_lite::JsonRpc;
use snowflake::ProcessUniqueId;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

// ZomeFnCallArgs to ZomeFnCall
impl ZomeFnCall {
    fn from_args(args: ZomeFnCallArgs) -> Self {
        ZomeFnCall::new(&args.zome_name, args.cap, &args.fn_name, args.fn_args)
    }
}

/// HcApiFuncIndex::CALL function code
/// args: [0] encoded MemoryAllocation as u64
/// expected complex argument: {zome_name: String, cap_token: Address, fn_name: String, args: String}
/// args from API call are converted into a ZomeFnCall
/// Launch an Action::Call with newly formed ZomeFnCall
/// Waits for a ZomeFnResult
/// Returns an HcApiReturnCode as I64
pub fn invoke_call(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let zome_call_data = runtime.zome_call_data()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let input = match ZomeFnCallArgs::try_from(args_str.clone()) {
        Ok(input) => input,
        // Exit on error
        Err(_) => {
            zome_call_data.context.log(format!(
                "err/zome: invoke_call failed to deserialize: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let result = if input.instance_handle == String::from(THIS_INSTANCE) {
        // ZomeFnCallArgs to ZomeFnCall
        let zome_call = ZomeFnCall::from_args(input.clone());

        // Don't allow recursive calls
        if zome_call.same_fn_as(&zome_call_data.zome_call) {
            return ribosome_error_code!(RecursiveCallForbidden);
        }
        local_call(runtime, input)
    } else {
        bridge_call(runtime, input)
    };

    runtime.store_result(result)
}

fn local_call(runtime: &mut Runtime, input: ZomeFnCallArgs) -> Result<JsonString, HolochainError> {
    let zome_call_data = runtime.zome_call_data().map_err(|_| {
        HolochainError::ErrorGeneric(
            "expecting zome call data in local call not null call".to_string(),
        )
    })?;
    // ZomeFnCallArgs to ZomeFnCall
    let zome_call = ZomeFnCall::from_args(input);
    let context = &zome_call_data.context;
    context.block_on(call_zome_function(zome_call, context))
}

fn bridge_call(runtime: &mut Runtime, input: ZomeFnCallArgs) -> Result<JsonString, HolochainError> {
    let zome_call_data = runtime.zome_call_data().map_err(|_| {
        HolochainError::ErrorGeneric(
            "expecting zome call data in bridge call not null call".to_string(),
        )
    })?;
    let conductor_api =
        zome_call_data
            .context
            .conductor_api
            .clone()
            .ok_or(HolochainError::ConfigError(
                "No conductor API in context".to_string(),
            ))?;

    let method = format!(
        "{}/{}/{}",
        input.instance_handle, input.zome_name, input.fn_name
    );

    let handler = conductor_api.write().unwrap();

    let id = ProcessUniqueId::new();
    let request = format!(
        r#"{{"jsonrpc": "2.0", "method": "{}", "params": {}, "id": "{}"}}"#,
        method, input.fn_args, id
    );

    let response = handler
        .handle_request_sync(&request)
        .ok_or("Bridge call failed".to_string())?;

    let response = JsonRpc::parse(&response)?;

    match response {
        JsonRpc::Success(_) => Ok(JsonString::from(
            serde_json::to_string(&response.get_result().unwrap()).unwrap(),
        )),
        JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(
            serde_json::to_string(&response.get_error().unwrap()).unwrap(),
        )),
        _ => Err(HolochainError::ErrorGeneric(
            "Bridge call failed".to_string(),
        )),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate test_utils;
    extern crate wabt;

    use crate::{
        context::Context,
        instance::{tests::test_instance_and_context, Instance},
        nucleus::{
            ribosome::{
                api::{
                    call::ZomeFnCall,
                    tests::{
                        test_function_name, test_parameters, test_zome_api_function_wasm,
                        test_zome_name,
                    },
                    ZomeApiFunction,
                },
                Defn,
            },
            tests::{test_capability_call, test_capability_name},
        },
        workflows::author_entry::author_entry,
    };
    use holochain_core_types::{
        cas::content::Address,
        dna::{
            capabilities::{Capability, CapabilityCall, CapabilityType},
            fn_declarations::FnDeclaration,
            Dna,
        },
        entry::{cap_entries::CapTokenGrant, Entry},
        error::{DnaError, HolochainError},
        json::JsonString,
    };
    use holochain_wasm_utils::api_serialization::ZomeFnCallArgs;

    use serde_json;
    use std::{
        collections::BTreeMap,
        sync::{mpsc::RecvTimeoutError, Arc},
    };
    use test_utils::create_test_dna_with_defs;

    /// dummy commit args from standard test entry
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_bad_args_bytes() -> Vec<u8> {
        let args = ZomeFnCallArgs {
            instance_handle: "instance_handle".to_string(),
            zome_name: "zome_name".to_string(),
            cap: Some(CapabilityCall::new(Address::from("bad cap_token"), None)),
            fn_name: "fn_name".to_string(),
            fn_args: "fn_args".to_string(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_args_bytes() -> Vec<u8> {
        let args = ZomeFnCallArgs {
            instance_handle: THIS_INSTANCE.to_string(),
            zome_name: test_zome_name(),
            cap: Some(test_capability_call()),
            fn_name: test_function_name(),
            fn_args: test_parameters(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    #[allow(dead_code)]
    struct TestSetup {
        context: Arc<Context>,
        instance: Instance,
    }

    fn setup_test(dna: Dna) -> TestSetup {
        let (instance, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");
        TestSetup {
            context: context,
            instance: instance,
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn test_reduce_call(
        test_setup: &TestSetup,
        token_str: &str,
        _caller: Address,
        expected: Result<Result<JsonString, HolochainError>, RecvTimeoutError>,
    ) {
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(CapabilityCall::new(Address::from(token_str), None)),
            "test",
            "{}",
        );

        let context = &test_setup.context;
        let result = context.block_on(call_zome_function(zome_call, context));
        assert_eq!(expected, Ok(result));
    }

    #[test]
    fn test_call_no_zome() {
        let dna = test_utils::create_test_dna_with_wat("bad_zome", &test_capability_name(), None);
        let test_setup = setup_test(dna);
        let expected = Ok(Err(HolochainError::Dna(DnaError::ZomeNotFound(
            r#"Zome 'test_zome' not found"#.to_string(),
        ))));
        test_reduce_call(&test_setup, "foo token", Address::from("caller"), expected);
    }

    fn setup_dna_for_cap_test(cap_type: CapabilityType) -> Dna {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::Call.as_str());
        let mut capability = Capability::new(cap_type);
        let fn_decl = FnDeclaration {
            name: test_function_name(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        };
        capability.functions = vec![fn_decl.name.clone()];
        let mut capabilities = BTreeMap::new();
        capabilities.insert(test_capability_name(), capability);
        let mut functions = Vec::new();
        functions.push(fn_decl);

        create_test_dna_with_defs(&test_zome_name(), (functions, capabilities), &wasm)
    }

    #[test]
    fn test_call_public() {
        let dna = setup_dna_for_cap_test(CapabilityType::Public);
        let test_setup = setup_test(dna);
        // Expecting error since there is no function in wasm to call
        let expected = Ok(Err(HolochainError::RibosomeFailed(
            "Zome function failure: Argument deserialization failed".to_string(),
        )));
        test_reduce_call(&test_setup, "", Address::from("caller"), expected);
    }

    #[test]
    fn test_call_transferable() {
        let dna = setup_dna_for_cap_test(CapabilityType::Transferable);
        let test_setup = setup_test(dna);
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));
        test_reduce_call(&test_setup, "", Address::from("caller"), expected_failure);

        // Expecting error since there is no function in wasm to call
        let expected = Ok(Err(HolochainError::RibosomeFailed(
            "Zome function failure: Argument deserialization failed".to_string(),
        )));
        let agent_token_str = test_setup.context.agent_id.key.clone();
        test_reduce_call(
            &test_setup,
            &agent_token_str,
            Address::from(agent_token_str.clone()),
            expected.clone(),
        );

        let grant = CapTokenGrant::create(CapabilityType::Transferable, None).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let addr = test_setup
            .context
            .block_on(author_entry(&grant_entry, None, &test_setup.context))
            .unwrap();
        test_reduce_call(
            &test_setup,
            &String::from(addr),
            Address::from("any caller"),
            expected,
        );
    }

    #[test]
    fn test_call_assigned() {
        let dna = setup_dna_for_cap_test(CapabilityType::Assigned);
        let test_setup = setup_test(dna);
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));
        test_reduce_call(
            &test_setup,
            "",
            Address::from("any caller"),
            expected_failure.clone(),
        );

        // Expecting error since there is no function in wasm to call
        let expected = Ok(Err(HolochainError::RibosomeFailed(
            "Zome function failure: Argument deserialization failed".to_string(),
        )));
        let agent_token_str = test_setup.context.agent_id.key.clone();
        test_reduce_call(
            &test_setup,
            &agent_token_str,
            Address::from(agent_token_str.clone()),
            expected.clone(),
        );

        let someone = Address::from("somoeone");
        let grant =
            CapTokenGrant::create(CapabilityType::Assigned, Some(vec![someone.clone()])).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let addr = test_setup
            .context
            .block_on(author_entry(&grant_entry, None, &test_setup.context))
            .unwrap();
        test_reduce_call(
            &test_setup,
            &String::from(addr.clone()),
            someone,
            expected.clone(),
        );

        /* function call doesn't know who the caller is yet so can't do the check in reduce
                let someone_else = Address::from("somoeone_else");
                test_reduce_call(&test_setup,&String::from(addr),someone_else, expected_failure.clone());
        */
    }
}
