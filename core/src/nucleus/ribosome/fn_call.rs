/// fn_call is the module that implements calling zome functions
///
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{ribosome, state::NucleusState, ExecuteZomeFnResponse, ZomeFnCall},
};
use holochain_core_types::{
    dna::{capabilities::CapabilityCall, wasm::DnaWasm},
    entry::cap_entries::CapTokenGrant,
    error::HolochainError,
};
use std::{convert::TryFrom, sync::Arc, thread};

/// Runs a zome function call in it's own thread if valid.  This function gets called by reducers,
/// either from externally exposed functions (via call_and_wit_for_result ),
/// or from internal calls from the zomes via the api invoke_call function.
pub fn do_call(
    context: Arc<Context>,
    state: &mut NucleusState,
    fn_call: ZomeFnCall,
) -> Result<(), HolochainError> {
    context.log(format!(
        "debug/reduce/do_call: Validating call: {:?}",
        fn_call
    ));
    // 1. Validate the call (a number of things could go wrong)
    let (dna_name, wasm) = validate_call(context.clone(), state, &fn_call)?;

    context.log(format!(
        "debug/reduce/do_call: executing call: {:?}",
        fn_call
    ));

    // 2. execute it in a separate thread
    state.zome_calls.insert(fn_call.clone(), None);

    thread::spawn(move || {
        // Have Ribosome spin up DNA and call the zome function
        let call_result = ribosome::run_dna(
            &dna_name,
            context.clone(),
            wasm.code,
            &fn_call,
            Some(fn_call.clone().parameters.into_bytes()),
        );
        // Construct response
        let response = ExecuteZomeFnResponse::new(fn_call.clone(), call_result);
        // Send ReturnZomeFunctionResult Action
        context
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(
                response,
            )))
            .expect("action channel to be open in reducer");
    });
    Ok(())
}

pub fn validate_call(
    context: Arc<Context>,
    state: &NucleusState,
    fn_call: &ZomeFnCall,
) -> Result<(String, DnaWasm), HolochainError> {
    // make sure the dna, zome and function exists and return pretty errors if they don't
    let dna = state.dna().ok_or_else(|| HolochainError::DnaMissing)?;
    let zome = dna
        .get_zome(&fn_call.zome_name)
        .map_err(|e| HolochainError::Dna(e))?;
    let _ = dna
        .get_function_with_zome_name(&fn_call.zome_name, &fn_call.fn_name)
        .map_err(|e| HolochainError::Dna(e))?;

    if !zome.is_fn_public(&fn_call.fn_name) && !check_capability(context.clone(), &fn_call.clone())
    {
        return Err(HolochainError::CapabilityCheckFailed);
    }
    Ok((dna.name.clone(), zome.code.clone()))
}

// TODO: check the signature too
fn is_token_the_agent(context: Arc<Context>, cap: &Option<CapabilityCall>) -> bool {
    match cap {
        None => false,
        Some(call) => context.agent_id.key == call.cap_token.to_string(),
    }
}

/// checks to see if a given function call is allowable according to the capabilities
/// that have been registered to callers in the chain.
fn check_capability(context: Arc<Context>, fn_call: &ZomeFnCall) -> bool {
    // the agent can always do everything
    if is_token_the_agent(context.clone(), &fn_call.cap) {
        return true;
    }

    match fn_call.cap.clone() {
        None => false,
        Some(cap_call) => {
            let chain = &context.chain_storage;
            let maybe_json = chain.read().unwrap().fetch(&cap_call.cap_token).unwrap();
            let grant = match maybe_json {
                Some(content) => CapTokenGrant::try_from(content).unwrap(),
                None => return false,
            };
            grant.verify(Some(&cap_call))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate test_utils;
    extern crate wabt;

    use crate::{
        action::{Action, ActionWrapper},
        context::Context,
        instance::{tests::*, Instance, Observer, RECV_DEFAULT_TIMEOUT_MS},
        nucleus::{
            ribosome::{
                api::{
                    tests::{
                        test_function_name, test_parameters, test_zome_api_function_wasm,
                        test_zome_name,
                    },
                    ZomeApiFunction,
                },
                Defn,
            },
            tests::{test_capability_name, *},
            ZomeFnCall,
        },
        workflows::author_entry::author_entry,
    };
    use holochain_core_types::{
        cas::content::Address,
        dna::{
            capabilities::{CallSignature, Capability, CapabilityCall, CapabilityType},
            fn_declarations::FnDeclaration,
            Dna,
        },
        entry::{cap_entries::CapTokenGrant, Entry},
        error::{DnaError, HolochainError},
        json::JsonString,
    };

    use futures::executor::block_on;
    use std::{
        collections::BTreeMap,
        sync::{
            mpsc::{channel, RecvTimeoutError},
            Arc,
        },
    };
    use test_utils::create_test_dna_with_defs;

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
        cap_call: Option<CapabilityCall>,
        expected: Result<Result<JsonString, HolochainError>, RecvTimeoutError>,
    ) {
        let zome_call = ZomeFnCall::new("test_zome", cap_call, "test", "{}");
        let zome_call_action = ActionWrapper::new(Action::Call(zome_call.clone()));

        // process the action
        let (sender, receiver) = channel();
        let closure = move |state: &crate::state::State| {
            // Observer waits for a ribosome_call_result
            let opt_res = state.nucleus().zome_call_result(&zome_call);
            match opt_res {
                Some(res) => {
                    // @TODO never panic in wasm
                    // @see https://github.com/holochain/holochain-rust/issues/159
                    sender
                        .send(res)
                        // the channel stays connected until the first message has been sent
                        // if this fails that means that it was called after having returned done=true
                        .expect("observer called after done");

                    true
                }
                None => false,
            }
        };

        let observer = Observer {
            sensor: Box::new(closure),
        };

        let mut state_observers: Vec<Observer> = Vec::new();
        state_observers.push(observer);
        let (_, rx_observer) = channel::<Observer>();
        test_setup.instance.process_action(
            zome_call_action,
            state_observers,
            &rx_observer,
            &test_setup.context,
        );

        let action_result = receiver.recv_timeout(RECV_DEFAULT_TIMEOUT_MS);

        assert_eq!(expected, action_result);
    }

    #[test]
    fn test_call_no_zome() {
        let dna = test_utils::create_test_dna_with_wat("bad_zome", &test_capability_name(), None);
        let test_setup = setup_test(dna);
        let expected = Ok(Err(HolochainError::Dna(DnaError::ZomeNotFound(
            r#"Zome 'test_zome' not found"#.to_string(),
        ))));
        test_reduce_call(&test_setup, None, expected);
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

    // success to test_reduce_call is when the function gets called which shows up as a
    // timeout error because the test wasm doesn't have any test functions defined.
    static SUCCESS_EXPECTED: Result<Result<JsonString, HolochainError>, RecvTimeoutError> =
        Err(RecvTimeoutError::Disconnected);

    #[test]
    fn test_call_public() {
        let dna = setup_dna_for_cap_test(CapabilityType::Public);
        let test_setup = setup_test(dna);
        // make the call with no capability call
        test_reduce_call(&test_setup, None, SUCCESS_EXPECTED.clone());
    }

    #[test]
    fn test_call_transferable() {
        let dna = setup_dna_for_cap_test(CapabilityType::Transferable);
        let test_setup = setup_test(dna);
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));

        // make the call with an invalid capability call, i.e. incorrect token
        let cap_call = CapabilityCall::new(
            Address::from("foo_token"),
            Address::from("some caller"),
            CallSignature {},
        );
        test_reduce_call(&test_setup, Some(cap_call), expected_failure);

        let agent_token_str = test_setup.context.agent_id.key.clone();
        let cap_call = CapabilityCall::new(
            Address::from(agent_token_str.clone()),
            Address::from(agent_token_str),
            CallSignature {},
        );

        test_reduce_call(&test_setup, Some(cap_call), SUCCESS_EXPECTED.clone());

        // make the call with an invalid capability call, i.e. correct token
        let grant = CapTokenGrant::create(CapabilityType::Transferable, None).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let addr = block_on(author_entry(&grant_entry, None, &test_setup.context)).unwrap();
        let cap_call = CapabilityCall::new(addr, Address::from("any caller"), CallSignature {});
        test_reduce_call(&test_setup, Some(cap_call), SUCCESS_EXPECTED.clone());
    }

    #[test]
    fn test_call_assigned() {
        let dna = setup_dna_for_cap_test(CapabilityType::Assigned);
        let test_setup = setup_test(dna);
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));
        let cap_call = CapabilityCall::new(
            Address::from("foo_token"),
            Address::from("any caller"),
            CallSignature {},
        );
        test_reduce_call(&test_setup, Some(cap_call), expected_failure.clone());

        // test assigned capability where the caller is the agent
        let agent_token_str = test_setup.context.agent_id.key.clone();
        let cap_call = CapabilityCall::new(
            Address::from(agent_token_str.clone()),
            Address::from(agent_token_str),
            CallSignature {},
        );
        test_reduce_call(&test_setup, Some(cap_call), SUCCESS_EXPECTED.clone());

        // test assigned capability where the caller is someone else
        let someone = Address::from("somoeone");
        let grant =
            CapTokenGrant::create(CapabilityType::Assigned, Some(vec![someone.clone()])).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let addr = block_on(author_entry(&grant_entry, None, &test_setup.context)).unwrap();
        let cap_call = CapabilityCall::new(addr, someone, CallSignature {});
        test_reduce_call(&test_setup, Some(cap_call), SUCCESS_EXPECTED.clone());

        /* function call doesn't know who the caller is yet so can't do the check in reduce
                let someone_else = Address::from("somoeone_else");
                test_reduce_call(&test_setup,&String::from(addr),someone_else, expected_failure.clone());
        */
    }

    #[test]
    fn test_agent_as_token() {
        let context = test_context("alice", None);
        let agent_token = Address::from(context.agent_id.key.clone());
        let cap_call = CapabilityCall::new(agent_token.clone(), agent_token, CallSignature {});
        assert!(is_token_the_agent(context.clone(), &Some(cap_call)));
        let cap_call = CapabilityCall::new(
            Address::from("fake_token"),
            Address::from("someone"),
            CallSignature {},
        );
        assert!(!is_token_the_agent(context, &Some(cap_call)));
    }
}
