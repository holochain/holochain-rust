use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        actions::get_entry::get_entry_from_agent_chain,
        ribosome::{self, capabilities::CapabilityRequest, WasmCallData},
        ZomeFnCall, ZomeFnResult,
    },
};
use holochain_core_types::{
    cas::content::Address,
    dna::wasm::DnaWasm,
    entry::{
        cap_entries::{CapTokenGrant, CapabilityType},
        Entry,
    },
    error::HolochainError,
    json::JsonString,
    signature::Signature,
};

use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use std::{pin::Pin, sync::Arc, thread};

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ExecuteZomeFnResponse {
    call: ZomeFnCall,
    result: ZomeFnResult,
}

impl ExecuteZomeFnResponse {
    pub fn new(call: ZomeFnCall, result: Result<JsonString, HolochainError>) -> Self {
        ExecuteZomeFnResponse { call, result }
    }

    /// read only access to call
    pub fn call(&self) -> ZomeFnCall {
        self.call.clone()
    }

    /// read only access to result
    pub fn result(&self) -> Result<JsonString, HolochainError> {
        self.result.clone()
    }
}

/// Execution of zome calls
/// This function is kicking off the execution of a given zome function with given parameters.
/// It dispatches two actions:
/// * `SignalZomeFunctionCall`: after passing checks and before actually starting the Ribosome,
/// * `ReturnZomeFunctionResult`: asynchronously after execution of the Ribosome has completed.
///
/// It is doing pre-checks (such as the capability check) synchronously but then spawns a new
/// thread to run the Ribosome in.
///
/// Being an async function, it returns a future that is polling the instance's State until
/// the call result gets added there through the `RetunrZomeFunctionResult` action.
///
/// Use Context::block_on to wait for the call result.
pub async fn call_zome_function(
    zome_call: ZomeFnCall,
    context: &Arc<Context>,
) -> Result<JsonString, HolochainError> {
    context.log(format!(
        "debug/actions/call_zome_fn: Validating call: {:?}",
        zome_call
    ));

    // 1. Validate the call (a number of things could go wrong)
    let (dna_name, wasm) = validate_call(context.clone(), &zome_call)?;

    context.log(format!(
        "debug/actions/call_zome_fn: executing call: {:?}",
        zome_call
    ));

    // Clone context and call data for the Ribosome thread
    let context_clone = context.clone();
    let zome_call_clone = zome_call.clone();

    // Signal (currently mainly to the nodejs_waiter) that we are about to start a zome function:
    context
        .action_channel()
        .send(ActionWrapper::new(Action::SignalZomeFunctionCall(
            zome_call.clone(),
        )))
        .expect("action channel to be open");

    let _ = thread::spawn(move || {
        // Have Ribosome spin up DNA and call the zome function
        let call_result = ribosome::run_dna(
            wasm.code,
            Some(zome_call_clone.clone().parameters.into_bytes()),
            WasmCallData::new_zome_call(
                context_clone.clone(),
                dna_name.clone(),
                zome_call_clone.clone(),
            ),
        );
        // Construct response
        let response = ExecuteZomeFnResponse::new(zome_call_clone, call_result);
        // Send ReturnZomeFunctionResult Action
        context_clone
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(
                response,
            )))
            .expect("action channel to be open in reducer");
    });

    await!(CallResultFuture {
        context: context.clone(),
        zome_call,
    })
}

/// validates that a given zome function call specifies a correct zome function and capability grant
pub fn validate_call(
    context: Arc<Context>,
    fn_call: &ZomeFnCall,
) -> Result<(String, DnaWasm), HolochainError> {
    let state = context.state().ok_or(HolochainError::ErrorGeneric(
        "Context not initialized".to_string(),
    ))?;

    let nucleus_state = state.nucleus();

    // make sure the dna, zome and function exists and return pretty errors if they don't
    let dna = nucleus_state
        .dna()
        .ok_or_else(|| HolochainError::DnaMissing)?;
    let zome = dna
        .get_zome(&fn_call.zome_name)
        .map_err(|e| HolochainError::Dna(e))?;
    let _ = dna
        .get_function_with_zome_name(&fn_call.zome_name, &fn_call.fn_name)
        .map_err(|e| HolochainError::Dna(e))?;

    if check_capability(context.clone(), fn_call)
        || (is_token_the_agent(context.clone(), &fn_call.cap)
            && verify_call_sig(
                context.clone(),
                &fn_call.cap.provenance.1,
                &fn_call.fn_name,
                fn_call.parameters.clone(),
            ))
    {
        Ok((dna.name.clone(), zome.code.clone()))
    } else {
        Err(HolochainError::CapabilityCheckFailed)
    }
}

fn is_token_the_agent(context: Arc<Context>, request: &CapabilityRequest) -> bool {
    context.agent_id.pub_sign_key == request.cap_token.to_string()
}

fn get_grant(context: &Arc<Context>, address: &Address) -> Option<CapTokenGrant> {
    match get_entry_from_agent_chain(context, address).ok()?? {
        Entry::CapTokenGrant(grant) => Some(grant),
        _ => None,
    }
}

/// checks to see if a given function call is allowable according to the capabilities
/// that have been registered to callers by looking for grants in the chain.
pub fn check_capability(context: Arc<Context>, fn_call: &ZomeFnCall) -> bool {
    let maybe_grant = get_grant(&context.clone(), &fn_call.cap_token());
    match maybe_grant {
        None => false,
        Some(grant) => verify_grant(context.clone(), &grant, fn_call),
    }
}

// temporary function to create a mock signature of for a zome call cap request
fn make_call_sig<J: Into<JsonString>>(
    context: Arc<Context>,
    function: &str,
    parameters: J,
) -> Signature {
    Signature::from(format!(
        "{}:{}:{}",
        context.agent_id.pub_sign_key,
        function,
        parameters.into()
    ))
}

// temporary function to verify a mock signature of for a zome call cap request
pub fn verify_call_sig<J: Into<JsonString>>(
    context: Arc<Context>,
    call_sig: &Signature,
    function: &str,
    parameters: J,
) -> bool {
    let mock_signature = Signature::from(format!(
        "{}:{}:{}",
        context.agent_id.pub_sign_key,
        function,
        parameters.into()
    ));
    *call_sig == mock_signature
}

/// creates a capability request for a zome call by signing the function name and parameters
pub fn make_cap_request_for_call<J: Into<JsonString>>(
    context: Arc<Context>,
    cap_token: Address,
    caller: Address,
    function: &str,
    parameters: J,
) -> CapabilityRequest {
    CapabilityRequest::new(
        cap_token,
        caller,
        make_call_sig(context, function, parameters),
    )
}

/// verifies that this grant is valid for a given requester and token value
pub fn verify_grant(context: Arc<Context>, grant: &CapTokenGrant, fn_call: &ZomeFnCall) -> bool {
    let cap_functions = grant.functions();
    let maybe_zome_grants = cap_functions.get(&fn_call.zome_name);
    if maybe_zome_grants.is_none() {
        context.log(format!(
            "debug/actions/verify_grant: no grant for zome {:?} in grant {:?}",
            fn_call.zome_name, cap_functions
        ));
        return false;
    }
    if !maybe_zome_grants.unwrap().contains(&fn_call.fn_name) {
        context.log(format!(
            "debug/actions/verify_grant: no grant for function {:?} in grant {:?}",
            fn_call.fn_name, maybe_zome_grants
        ));
        return false;
    }

    if grant.token() != fn_call.cap_token() {
        context.log(format!(
            "debug/actions/verify_grant: grant token doesn't match: expecting {:?} got {:?}",
            grant.token(),
            fn_call.cap_token()
        ));
        return false;
    }

    if !verify_call_sig(
        context.clone(),
        &fn_call.cap.provenance.1,
        &fn_call.fn_name,
        fn_call.parameters.clone(),
    ) {
        context.log("debug/actions/verify_grant: call signature did not match");
        return false;
    }

    match grant.cap_type() {
        CapabilityType::Public => true,
        CapabilityType::Transferable => true,
        CapabilityType::Assigned => {
            // unwraps are safe because type comes from the shape of
            // the assignee, and the from must some by the check above.
            if !grant
                .assignees()
                .unwrap()
                .contains(&fn_call.cap.provenance.0)
            {
                context.log("debug/actions/verify_grant: caller not one of the assignees");
                return false;
            }
            true
        }
    }
}

/// CallResultFuture resolves to an Result<JsonString, HolochainError>.
/// Tracks the nucleus State, waiting for a result to the given zome function call to appear.
pub struct CallResultFuture {
    context: Arc<Context>,
    zome_call: ZomeFnCall,
}

impl Future for CallResultFuture {
    type Output = Result<JsonString, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        // With our own executor implementation in Context::block_on we actually
        // wouldn't need the waker since this executor is attached to the redux loop
        // and re-polls after every State mutation.
        // Leaving this in to be safe against running this future in another executor.
        lw.wake();

        if let Some(state) = self.context.state() {
            match state.nucleus().zome_call_result(&self.zome_call) {
                Some(result) => Poll::Ready(result),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        context::Context,
        instance::tests::*,
        nucleus::{actions::tests::test_dna, ribosome::capabilities::CapabilityRequest, tests::*},
        workflows::author_entry::author_entry,
    };
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::Address,
        entry::{
            cap_entries::{CapFunctions, CapTokenGrant, CapabilityType},
            Entry,
        },
        signature::Signature,
    };

    #[test]
    fn test_agent_as_token() {
        let context = test_context("alice", None);
        let agent_token = Address::from(context.agent_id.pub_sign_key.clone());
        let cap_request = make_cap_request_for_call(
            context.clone(),
            agent_token.clone(),
            agent_token.clone(),
            "test",
            "{}",
        );
        assert!(is_token_the_agent(context.clone(), &cap_request));

        // bogus token should fail
        let cap_request = CapabilityRequest::new(
            Address::from("fake_token"),
            Address::from("someone"),
            Signature::fake(),
        );
        assert!(!is_token_the_agent(context, &cap_request));
    }

    #[test]
    fn test_call_signatures() {
        let context1 = test_context("alice", None);
        let context2 = test_context("bob", None);

        // only exact same call signed by the same person should verify
        let call_sig1 = make_call_sig(context1.clone(), "func", "{}");
        assert!(verify_call_sig(context1.clone(), &call_sig1, "func", "{}"));
        assert!(!verify_call_sig(
            context1.clone(),
            &call_sig1,
            "func1",
            "{}"
        ));
        assert!(!verify_call_sig(context1, &call_sig1, "func", "{\"x\":1}"));

        assert!(!verify_call_sig(context2.clone(), &call_sig1, "func", "{}"));
        assert!(!verify_call_sig(
            context2.clone(),
            &call_sig1,
            "func1",
            "{}"
        ));
        assert!(!verify_call_sig(context2, &call_sig1, "func", "{\"x\":1}"));
    }

    #[test]
    fn test_make_cap_request_for_call() {
        let context = test_context("alice", None);
        let cap_request = make_cap_request_for_call(
            context.clone(),
            dummy_capability_token(),
            Address::from("caller"),
            "some_fn",
            "{}",
        );
        assert_eq!(cap_request.cap_token, dummy_capability_token());
        assert_eq!(cap_request.provenance.0, Address::from("caller"));
        assert_eq!(
            cap_request.provenance.1,
            make_call_sig(context, "some_fn", "{}")
        );
    }

    #[test]
    fn test_get_grant() {
        let dna = test_dna();
        let (_, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");

        let mut cap_functions = CapFunctions::new();
        cap_functions.insert("test_zome".to_string(), vec![String::from("test")]);
        let grant =
            CapTokenGrant::create(CapabilityType::Transferable, None, cap_functions).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant.clone());
        let grant_addr = block_on(author_entry(&grant_entry, None, &context)).unwrap();
        let maybe_grant = get_grant(&context, &grant_addr);
        assert_eq!(maybe_grant, Some(grant));
    }

    #[test]
    fn test_verify_grant() {
        let context = test_context("alice", None);
        let test_address1 = Address::from("agent 1");
        let test_address2 = Address::from("some other identity");

        fn zome_call_valid(context: Arc<Context>, token: &Address, addr: &Address) -> ZomeFnCall {
            ZomeFnCall::new(
                "test_zome",
                make_cap_request_for_call(
                    context.clone(),
                    token.clone(),
                    addr.clone(),
                    "test",
                    "{}",
                ),
                "test",
                "{}",
            )
        }

        let zome_call_from_addr1_bad_token = &ZomeFnCall::new(
            "test_zome",
            make_cap_request_for_call(
                context.clone(),
                Address::from("bad token"),
                test_address1.clone(),
                "test",
                "{}",
            ),
            "test",
            "{}",
        );

        let mut cap_functions = CapFunctions::new();
        cap_functions.insert("test_zome".to_string(), vec![String::from("test")]);

        let grant = CapTokenGrant::create(CapabilityType::Public, None, cap_functions).unwrap();
        let token = grant.token();
        assert!(verify_grant(
            context.clone(),
            &grant,
            &zome_call_valid(context.clone(), &token, &test_address1)
        ));
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_token
        ));

        let mut cap_functions = CapFunctions::new();
        cap_functions.insert("test_zome".to_string(), vec![String::from("other_fn")]);
        let grant_for_other_fn =
            CapTokenGrant::create(CapabilityType::Transferable, None, cap_functions).unwrap();
        assert!(!verify_grant(
            context.clone(),
            &grant_for_other_fn,
            &zome_call_valid(context.clone(), &grant_for_other_fn.token(), &test_address1)
        ));

        let mut cap_functions = CapFunctions::new();
        cap_functions.insert("test_zome".to_string(), vec![String::from("test")]);
        let grant =
            CapTokenGrant::create(CapabilityType::Transferable, None, cap_functions).unwrap();

        let token = grant.token();
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_token
        ));

        // call with cap_request for a different function than the zome call
        let zome_call_from_addr1_bad_cap_request = &ZomeFnCall::new(
            "test_zome",
            make_cap_request_for_call(
                context.clone(),
                token.clone(),
                test_address1.clone(),
                "foo-fn",
                "{}",
            ),
            "test",
            "{}",
        );
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_cap_request
        ));

        assert!(verify_grant(
            context.clone(),
            &grant,
            &zome_call_valid(context.clone(), &token, &test_address1)
        ));
        // should work with same token from a different adddress
        assert!(verify_grant(
            context.clone(),
            &grant,
            &zome_call_valid(context.clone(), &token, &test_address2)
        ));

        let mut cap_functions = CapFunctions::new();
        cap_functions.insert("test_zome".to_string(), vec![String::from("test")]);
        let grant = CapTokenGrant::create(
            CapabilityType::Assigned,
            Some(vec![test_address1.clone()]),
            cap_functions,
        )
        .unwrap();
        let token = grant.token();
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_token
        ));

        // call with cap_request for a different function than the zome call
        let zome_call_from_addr1_bad_cap_request = &ZomeFnCall::new(
            "test_zome",
            make_cap_request_for_call(
                context.clone(),
                token.clone(),
                test_address1.clone(),
                "foo-fn",
                "{}",
            ),
            "test",
            "{}",
        );
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_cap_request
        ));

        assert!(verify_grant(
            context.clone(),
            &grant,
            &zome_call_valid(context.clone(), &token, &test_address1)
        ));
        // should NOT work with same token from a different adddress
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_valid(context.clone(), &token, &test_address2)
        ));
    }
}
