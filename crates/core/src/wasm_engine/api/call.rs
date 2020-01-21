use crate::{
    context::Context,
    nucleus::{
        actions::call_zome_function::{call_zome_function, make_cap_request_for_call},
        ZomeFnCall,
    },
    wasm_engine::{api::ZomeApiResult, Runtime},
};
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use holochain_logging::prelude::*;

use holochain_wasm_utils::api_serialization::{ZomeFnCallArgs, THIS_INSTANCE};
use jsonrpc_lite::JsonRpc;
use snowflake::ProcessUniqueId;
use std::{convert::TryFrom, sync::Arc};
use wasmi::{RuntimeArgs, RuntimeValue};

// ZomeFnCallArgs to ZomeFnCall
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl ZomeFnCall {
    fn from_args(context: Arc<Context>, args: ZomeFnCallArgs) -> Self {
        // TODO we are currently signing the call ourself.  This signature
        // should have happend at the client and be extracted from the args.
        let cap_call = make_cap_request_for_call(
            context,
            args.cap_token,
            &args.fn_name,
            JsonString::from_json(&args.fn_args.clone()),
        );
        ZomeFnCall::new(
            &args.zome_name,
            cap_call,
            &args.fn_name,
            JsonString::from_json(&args.fn_args),
        )
    }
}

/// HcApiFuncIndex::CALL function code
/// args: [0] encoded MemoryAllocation as u64
/// expected complex argument: {zome_name: String, cap_token: Address, fn_name: String, args: String}
/// args from API call are converted into a ZomeFnCall
/// Launch an Action::Call with newly formed ZomeFnCall-
/// Waits for a ZomeFnResult
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_call(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let input = match ZomeFnCallArgs::try_from(args_str.clone()) {
        Ok(input) => input,
        // Exit on error
        Err(_) => {
            log_error!(
                context,
                "zome: invoke_call failed to deserialize: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let result = if input.instance_handle == THIS_INSTANCE {
        // ZomeFnCallArgs to ZomeFnCall
        let zome_call = ZomeFnCall::from_args(context.clone(), input.clone());

        if let Ok(zome_call_data) = runtime.zome_call_data() {
            // Don't allow recursive calls
            if zome_call.same_fn_as(&zome_call_data.call) {
                return ribosome_error_code!(RecursiveCallForbidden);
            }
        }
        local_call(runtime, input.clone()).map_err(|error| {
            log_error!(context, "zome-to-zome-call/[{:?}]: {:?}", input, error);
            error
        })
    } else {
        bridge_call(runtime, input.clone()).map_err(|error| {
            log_error!(context, "bridge-call/[{:?}]: {:?}", input, error);
            error
        })
    };

    runtime.store_result(result)
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn local_call(runtime: &mut Runtime, input: ZomeFnCallArgs) -> Result<JsonString, HolochainError> {
    let context = runtime.context().map_err(|_| {
        HolochainError::ErrorGeneric(
            "expecting zome call data in local call not null call".to_string(),
        )
    })?;
    // ZomeFnCallArgs to ZomeFnCall
    let zome_call = ZomeFnCall::from_args(context.clone(), input.clone());
    log_debug!(context, "blocking on zome call: {:?}", input.clone());
    let result = context.block_on(call_zome_function(zome_call, context.clone()));
    log_debug!(
        context,
        "blocked on zome call: {:?} with result {:?}",
        input,
        result
    );
    result
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn bridge_call(runtime: &mut Runtime, input: ZomeFnCallArgs) -> Result<JsonString, HolochainError> {
    let context = runtime.context().map_err(|_| {
        HolochainError::ErrorGeneric(
            "expecting zome call data in bridge call not null call".to_string(),
        )
    })?;
    let conductor_api = context.conductor_api.clone();

    let params = format!(
        r#"{{"instance_id":"{}", "zome": "{}", "function": "{}", "args": {}}}"#,
        input.instance_handle, input.zome_name, input.fn_name, input.fn_args
    );

    let handler = conductor_api.get().write().unwrap();

    let id = ProcessUniqueId::new();
    // json-rpc format
    let request = format!(
        r#"{{"jsonrpc": "2.0", "method": "call", "params": {}, "id": "{}"}}"#,
        params, id
    );

    let response = handler
        .handle_request_sync(&request)
        .ok_or("Bridge call failed")?;

    let response = JsonRpc::parse(&response)?;

    match response {
        JsonRpc::Success(_) => {
            // First we try to unwrap a potential stringification:
            let value_response = response.get_result().unwrap().to_owned();
            let string_response = value_response.to_string();
            let maybe_parsed_string: Result<String, serde_json::error::Error> =
                serde_json::from_str(&string_response);
            let sanitized_response = match maybe_parsed_string {
                Ok(string) => string,
                Err(_) => string_response,
            };
            // Below, sanitized_response is the same response but guaranteed without quotes.
            // This should be returned as a JsonString for handling in the zome code.
            Ok(JsonString::from_json(&sanitized_response))
        }
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
    use test_utils;

    use crate::{
        context::Context,
        instance::{
            tests::{test_context, test_instance_and_context},
            Instance,
        },
        nucleus::{
            actions::call_zome_function::{check_capability, validate_call},
            tests::*,
        },
        wasm_engine::{
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
        workflows::author_entry::author_entry,
    };
    use crossbeam_channel::RecvTimeoutError;
    use holochain_core_types::{
        dna::{
            capabilities::CapabilityRequest,
            fn_declarations::{FnDeclaration, TraitFns},
            traits::ReservedTraitNames,
            Dna,
        },
        entry::{
            cap_entries::{CapFunctions, CapTokenGrant, CapabilityType},
            Entry,
        },
        error::{DnaError, HolochainError},
        signature::Signature,
    };
    use holochain_json_api::json::JsonString;
    use holochain_persistence_api::cas::content::{Address, AddressableContent};
    use holochain_wasm_utils::api_serialization::ZomeFnCallArgs;
    use serde_json;
    use std::{collections::BTreeMap, sync::Arc};
    use test_utils::create_test_dna_with_defs;

    /// dummy commit args from standard test entry
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_bad_args_bytes() -> Vec<u8> {
        let args = ZomeFnCallArgs {
            instance_handle: "instance_handle".to_string(),
            zome_name: "zome_name".to_string(),
            cap_token: Address::from("bad cap_token"),
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
            cap_token: Address::from("test_token"),
            fn_name: test_function_name(),
            fn_args: test_parameters().to_string(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    #[allow(dead_code)]
    pub struct TestSetup {
        pub context: Arc<Context>,
        pub instance: Instance,
    }

    pub fn setup_test(dna: Dna, netname: &str) -> TestSetup {
        let netname = Some(netname);
        let (instance, context) =
            test_instance_and_context(dna, netname).expect("Could not initialize test instance");
        TestSetup { context, instance }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn test_reduce_call(
        test_setup: &TestSetup,
        cap_request: CapabilityRequest,
        expected: Result<Result<JsonString, HolochainError>, RecvTimeoutError>,
    ) {
        let zome_call = ZomeFnCall::new("test_zome", cap_request, "test", "{}");
        let result = test_setup
            .context
            .block_on(call_zome_function(zome_call, test_setup.context.clone()));
        assert_eq!(expected, Ok(result));
    }

    #[test]
    fn test_call_no_zome() {
        let dna = test_utils::create_test_dna_with_wat("bad_zome", None);
        let test_setup = setup_test(dna, "test_call_no_zome");
        let expected = Ok(Err(HolochainError::Dna(DnaError::ZomeNotFound(
            r#"Zome 'test_zome' not found"#.to_string(),
        ))));
        test_reduce_call(&test_setup, dummy_capability_request(), expected);
    }

    fn setup_dna_for_test(make_public: bool) -> Dna {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::Call.as_str());
        let mut trait_fns = TraitFns::new();
        let fn_decl = FnDeclaration {
            name: test_function_name(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        };
        trait_fns.functions = vec![fn_decl.name.clone()];
        let mut traits = BTreeMap::new();
        let trait_name = if make_public {
            ReservedTraitNames::Public.as_str().to_string()
        } else {
            "test_trait".to_string()
        };
        traits.insert(trait_name, trait_fns);
        let mut functions = Vec::new();
        functions.push(fn_decl);

        create_test_dna_with_defs(&test_zome_name(), (functions, traits), &wasm)
    }

    // success to test_reduce_call is when the function gets called which shows up as an
    // argument deserialization error because we are reusing the wasm from test_zome_api_function
    // which just passes the function parameter through to "invoke_call" which expects a
    // ZomeFnCallArgs struct which the test "{}" is not!
    // TODO: fix this bit of crazyness
    fn success_expected() -> Result<Result<JsonString, HolochainError>, RecvTimeoutError> {
        Ok(Err(HolochainError::RibosomeFailed(
            "Zome function failure: Argument deserialization failed".to_string(),
        )))
    }

    #[test]
    fn test_call_public() {
        let dna = setup_dna_for_test(true);
        let test_setup = setup_test(dna, "test_call_public");
        let token = test_setup.context.get_public_token().unwrap();
        let other_agent_context = test_context("other agent", None);
        let cap_request = make_cap_request_for_call(other_agent_context, token, "test", "{}");

        // make the call with public token capability call
        test_reduce_call(&test_setup, cap_request, success_expected());

        // make the call with a bogus public token capability call
        let cap_request = CapabilityRequest::new(
            Address::from("foo_token"),
            Address::from("some caller"),
            Signature::fake(),
        );
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));
        test_reduce_call(&test_setup, cap_request, expected_failure);
    }

    #[test]
    fn test_call_transferable() {
        let dna = setup_dna_for_test(false);
        let test_setup = setup_test(dna, "test_call_transferable");
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));

        // make the call with an invalid capability call, i.e. incorrect token
        let cap_request = CapabilityRequest::new(
            Address::from("foo_token"),
            Address::from("some caller"),
            Signature::fake(),
        );
        test_reduce_call(&test_setup, cap_request.clone(), expected_failure.clone());

        // make the call with an valid capability call from self
        let cap_request = test_agent_capability_request(test_setup.context.clone(), "test", "{}");
        test_reduce_call(&test_setup, cap_request, success_expected());

        // make the call with an invalid valid capability call from self
        let cap_request =
            test_agent_capability_request(test_setup.context.clone(), "some_fn", "{}");
        test_reduce_call(&test_setup, cap_request, expected_failure);

        let mut cap_functions = CapFunctions::new();
        cap_functions.insert("test_zome".to_string(), vec![String::from("test")]);
        // make the call with an valid capability call from a different sources
        let grant = CapTokenGrant::create("foo", CapabilityType::Transferable, None, cap_functions)
            .unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let addr = test_setup
            .context
            .block_on(author_entry(
                &grant_entry,
                None,
                &test_setup.context,
                &vec![],
            ))
            .unwrap()
            .address();
        let other_agent_context = test_context("other agent", None);
        let cap_request =
            make_cap_request_for_call(other_agent_context.clone(), addr, "test", "{}");
        test_reduce_call(&test_setup, cap_request, success_expected());
    }

    #[test]
    fn test_call_assigned() {
        let dna = setup_dna_for_test(false);
        let test_setup = setup_test(dna, "test_call_assigned");
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));
        let cap_request = CapabilityRequest::new(
            Address::from("foo_token"),
            Address::from("any caller"),
            Signature::fake(),
        );
        test_reduce_call(&test_setup, cap_request, expected_failure.clone());

        // test assigned capability where the caller is the agent
        let agent_token_str = test_setup.context.agent_id.address();
        let cap_request = make_cap_request_for_call(
            test_setup.context.clone(),
            Address::from(agent_token_str.clone()),
            "test",
            "{}",
        );
        test_reduce_call(&test_setup, cap_request, success_expected());

        // test assigned capability where the caller is someone else
        let other_agent_context = test_context("other agent", None);
        let someone = other_agent_context.agent_id.address();
        let mut cap_functions = CapFunctions::new();
        cap_functions.insert("test_zome".to_string(), vec![String::from("test")]);
        let grant = CapTokenGrant::create(
            "foo",
            CapabilityType::Assigned,
            Some(vec![someone.clone()]),
            cap_functions,
        )
        .unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let grant_addr = test_setup
            .context
            .block_on(author_entry(
                &grant_entry,
                None,
                &test_setup.context,
                &vec![],
            ))
            .unwrap()
            .address();
        let cap_request = make_cap_request_for_call(
            test_context("random other agent", None),
            grant_addr.clone(),
            "test",
            "{}",
        );
        test_reduce_call(&test_setup, cap_request, expected_failure.clone());

        // test assigned capability where the caller is someone else
        let cap_request =
            make_cap_request_for_call(other_agent_context.clone(), grant_addr, "test", "{}");
        test_reduce_call(&test_setup, cap_request, success_expected());
    }

    #[test]
    fn test_validate_call_public() {
        let dna = setup_dna_for_test(true);
        let test_setup = setup_test(dna, "test_validate_call_public");
        let context = test_setup.context;

        // non existent functions should fail
        let zome_call = ZomeFnCall::new("test_zome", dummy_capability_request(), "foo_func", "{}");
        let result = validate_call(context.clone(), &zome_call);
        assert_eq!(
            result,
            Err(HolochainError::Dna(DnaError::ZomeFunctionNotFound(
                String::from("Zome function \'foo_func\' not found in Zome \'test_zome\'")
            )))
        );

        // non existent zomes should fial
        let zome_call = ZomeFnCall::new("foo_zome", dummy_capability_request(), "test", "{}");
        let result = validate_call(context.clone(), &zome_call);
        assert_eq!(
            result,
            Err(HolochainError::Dna(DnaError::ZomeNotFound(String::from(
                "Zome \'foo_zome\' not found"
            ))))
        );
    }

    #[test]
    fn test_validate_call_by_agent() {
        let dna = setup_dna_for_test(false);
        let test_setup = setup_test(dna, "validate_call_by_agent");
        let context = test_setup.context;

        // non public call should fail
        let zome_call = ZomeFnCall::new("test_zome", dummy_capability_request(), "test", "{}");
        let result = validate_call(context.clone(), &zome_call);
        assert_eq!(result, Err(HolochainError::CapabilityCheckFailed));

        // if the agent doesn't correctly sign the call it should fail
        let zome_call = ZomeFnCall::new(
            "test_zome",
            make_cap_request_for_call(
                context.clone(),
                Address::from(context.agent_id.address()), // <- agent token
                "foo_function",                            //<- not the function in the zome_call!
                "{}",
            ),
            "test",
            "{}",
        );

        let result = validate_call(context.clone(), &zome_call);
        assert_eq!(result, Err(HolochainError::CapabilityCheckFailed));

        // should work with correctly signed cap_request
        let zome_call = ZomeFnCall::new(
            "test_zome",
            make_cap_request_for_call(
                context.clone(),
                Address::from(context.agent_id.address()), // <- agent token
                "test",
                "{}",
            ),
            "test",
            "{}",
        );
        let result = validate_call(context.clone(), &zome_call);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_capability_transferable() {
        let dna = setup_dna_for_test(false);
        let test_setup = setup_test(dna, "test_check_cap_transferable");
        let context = test_setup.context;

        // bogus cap_request should fail
        let zome_call = ZomeFnCall::new(
            "test_zome",
            CapabilityRequest::new(
                Address::from("foo_token"),
                Address::from("some caller"),
                Signature::fake(),
            ),
            "test",
            "{}",
        );
        assert!(!check_capability(context.clone(), &zome_call));

        let mut cap_functions = CapFunctions::new();
        cap_functions.insert("test_zome".to_string(), vec![String::from("test")]);
        // add the transferable grant and get the token which is the grant's address
        let grant = CapTokenGrant::create("foo", CapabilityType::Transferable, None, cap_functions)
            .unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let grant_addr = context
            .block_on(author_entry(&grant_entry, None, &context, &vec![]))
            .unwrap()
            .address();

        // make the call with a valid capability call from a random source should succeed
        let zome_call = ZomeFnCall::new(
            "test_zome",
            make_cap_request_for_call(
                test_context("some_random_agent", None),
                grant_addr,
                "test",
                "{}",
            ),
            "test",
            "{}",
        );
        assert!(check_capability(context.clone(), &zome_call));
    }
}
