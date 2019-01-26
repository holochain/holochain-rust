/// fn_call is the module that implements calling zome functions
///
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::{dispatch_action_with_observer, Observer},
    nucleus::{actions::get_entry::get_entry_from_agent_chain, ribosome, state::NucleusState},
};
use holochain_core_types::{
    cas::content::Address,
    dna::{
        capabilities::{CallSignature, CapabilityCall, CapabilityType},
        wasm::DnaWasm,
    },
    entry::{cap_entries::CapTokenGrant, Entry},
    error::{HcResult, HolochainError},
    json::JsonString,
    signature::Signature,
};
use snowflake;
use std::{
    sync::{
        mpsc::{sync_channel, SyncSender},
        Arc,
    },
    thread,
};

/// Struct holding data for requesting the execution of a Zome function (ExecuteZomeFunction Action)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZomeFnCall {
    id: snowflake::ProcessUniqueId,
    pub zome_name: String,
    pub cap: Option<CapabilityCall>,
    pub fn_name: String,

    pub parameters: JsonString,
}

impl ZomeFnCall {
    pub fn new<J: Into<JsonString>>(
        zome: &str,
        cap: Option<CapabilityCall>,
        function: &str,
        parameters: J,
    ) -> Self {
        ZomeFnCall {
            // @TODO can we defer to the ActionWrapper id?
            // @see https://github.com/holochain/holochain-rust/issues/198
            id: snowflake::ProcessUniqueId::new(),
            zome_name: zome.to_string(),
            cap: cap,
            fn_name: function.to_string(),
            parameters: parameters.into(),
        }
    }

    pub fn same_fn_as(&self, fn_call: &ZomeFnCall) -> bool {
        self.zome_name == fn_call.zome_name
            && self.cap == fn_call.cap
            && self.fn_name == fn_call.fn_name
    }

    pub fn cap_token(&self) -> Address {
        match self.cap.clone() {
            Some(call) => call.cap_token,
            None => panic!("null cap call unimplemented!"),
        }
    }
}

/// Reduce ExecuteZomeFunction Action
/// Execute an exposed Zome function in a separate thread and send the result in
/// a ReturnZomeFunctionResult Action on success or failure
pub(crate) fn reduce_execute_zome_function(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    fn dispatch_error_result(
        action_channel: &SyncSender<ActionWrapper>,
        fn_call: &ZomeFnCall,
        error: HolochainError,
    ) {
        let zome_not_found_response =
            ExecuteZomeFnResponse::new(fn_call.clone(), Err(error.clone()));

        action_channel
            .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(
                zome_not_found_response,
            )))
            .expect("action channel to be open in reducer");
    }

    let fn_call = match action_wrapper.action().clone() {
        Action::ExecuteZomeFunction(call) => call,
        _ => unreachable!(),
    };

    if let Some(err) = do_call(context.clone(), state, fn_call.clone()).err() {
        dispatch_error_result(context.action_channel(), &fn_call, err);
    }
}

/// Reduce ReturnZomeFunctionResult Action.
/// Simply drops function call into zome_calls state.
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
pub(crate) fn reduce_return_zome_function_result(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let fr = unwrap_to!(action => Action::ReturnZomeFunctionResult);
    // @TODO store the action and result directly
    // @see https://github.com/holochain/holochain-rust/issues/198
    state.zome_calls.insert(fr.call(), Some(fr.result()));
}

/// Dispatch ExecuteZoneFunction to and block until call has finished.
pub fn call_zome_and_wait_for_result(
    call: ZomeFnCall,
    action_channel: &SyncSender<ActionWrapper>,
    observer_channel: &SyncSender<Observer>,
) -> Result<JsonString, HolochainError> {
    let call_action_wrapper = ActionWrapper::new(Action::ExecuteZomeFunction(call.clone()));

    // Dispatch action with observer closure that waits for a result in the state
    let (sender, receiver) = sync_channel(1);
    dispatch_action_with_observer(
        action_channel,
        observer_channel,
        call_action_wrapper,
        move |state: &crate::state::State| {
            if let Some(result) = state.nucleus().zome_call_result(&call) {
                sender
                    .send(result.clone())
                    .expect("local channel to be open");
                true
            } else {
                false
            }
        },
    );
    // Block until we got that result through the channel:
    receiver.recv().expect("local channel to work")
}

/// Dispatch ExecuteZoneFunction to Instance and block until call has finished.
/// for test only?? <-- (apparently not, since it's used in Holochain::call)
pub fn call_and_wait_for_result(
    call: ZomeFnCall,
    instance: &mut crate::instance::Instance,
) -> Result<JsonString, HolochainError> {
    let call_action = ActionWrapper::new(Action::ExecuteZomeFunction(call.clone()));

    // Dispatch action with observer closure that waits for a result in the state
    let (sender, receiver) = sync_channel(1);
    instance.dispatch_with_observer(call_action, move |state: &crate::state::State| {
        if let Some(result) = state.nucleus().zome_call_result(&call) {
            sender
                .send(result.clone())
                .expect("local channel to be open");
            true
        } else {
            // @TODO: Use futures for this, and in case this should probably have a timeout
            false
        }
    });

    // Block until we got that result through the channel:
    receiver.recv().expect("local channel to work")
}

pub type ZomeFnResult = HcResult<JsonString>;

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

    // call is valid if the call is listed in a public capability OR
    // the capability check of the grant passes OR
    // or the token is the agent AND the call signature is valid.
    if zome.is_fn_public(&fn_call.fn_name)
        || match fn_call.cap.clone() {
            Some(cap) => {
                check_capability(context.clone(), fn_call)
                    || (is_token_the_agent(context.clone(), &cap)
                        && verify_call_sig(
                            context.clone(),
                            &cap.signature,
                            &fn_call.fn_name,
                            fn_call.parameters.clone(),
                        ))
            }
            None => false,
        }
    {
        return Ok((dna.name.clone(), zome.code.clone()));
    }
    Err(HolochainError::CapabilityCheckFailed)
}

fn is_token_the_agent(context: Arc<Context>, cap_call: &CapabilityCall) -> bool {
    context.agent_id.key == cap_call.cap_token.to_string()
}

fn get_grant(context: Arc<Context>, address: &Address) -> Option<CapTokenGrant> {
    match get_entry_from_agent_chain(&context, address).ok()?? {
        Entry::CapTokenGrant(grant) => Some(grant),
        _ => None,
    }
}

/// checks to see if a given function call is allowable according to the capabilities
/// that have been registered to callers by looking for grants in the chain.
fn check_capability(context: Arc<Context>, fn_call: &ZomeFnCall) -> bool {
    if fn_call.cap.is_none() {
        return false;
    }
    let cap_call = fn_call.cap.clone().unwrap();
    let maybe_grant = get_grant(context.clone(), &cap_call.cap_token);
    match maybe_grant {
        None => false,
        Some(grant) => verify_grant(context.clone(), &grant, fn_call),
    }
}

fn make_call_sig<J: Into<JsonString>>(
    context: Arc<Context>,
    function: &str,
    parameters: J,
) -> CallSignature {
    let mock_signature = Signature::from(format!(
        "{}:{}:{}",
        context.agent_id.key,
        function,
        parameters.into()
    ));
    CallSignature::new(mock_signature)
}

pub fn verify_call_sig<J: Into<JsonString>>(
    context: Arc<Context>,
    call_sig: &CallSignature,
    function: &str,
    parameters: J,
) -> bool {
    let mock_signature = Signature::from(format!(
        "{}:{}:{}",
        context.agent_id.key,
        function,
        parameters.into()
    ));
    call_sig.signature() == mock_signature
}

pub fn make_cap_call<J: Into<JsonString>>(
    context: Arc<Context>,
    cap_token: Address,
    caller: Address,
    function: &str,
    parameters: J,
) -> CapabilityCall {
    CapabilityCall::new(
        cap_token,
        caller,
        make_call_sig(context, function, parameters),
    )
}

/// verifies that this grant is valid for a given requester and token value
pub fn verify_grant(context: Arc<Context>, grant: &CapTokenGrant, fn_call: &ZomeFnCall) -> bool {
    let cap_type = grant.cap_type();
    if cap_type == CapabilityType::Public {
        return true;
    }
    if fn_call.cap.is_none() {
        return false;
    }
    let cap_call = fn_call.cap.clone().unwrap();

    if grant.token() != cap_call.cap_token {
        return false;
    }

    if !verify_call_sig(
        context.clone(),
        &cap_call.signature,
        &fn_call.fn_name,
        fn_call.parameters.clone(),
    ) {
        return false;
    }

    match grant.cap_type() {
        CapabilityType::Public => true,
        CapabilityType::Transferable => true,
        CapabilityType::Assigned => {
            // unwraps are safe because type comes from the shape of
            // the assignee, and the from must some by the check above.
            if !grant.assignees().unwrap().contains(&cap_call.caller) {
                return false;
            }
            true
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate test_utils;
    extern crate wabt;

    use crate::{
        action::{tests::test_action_wrapper_rzfr, Action, ActionWrapper},
        context::Context,
        instance::{tests::*, Instance, Observer, RECV_DEFAULT_TIMEOUT_MS},
        nucleus::{
            reduce,
            ribosome::{
                api::{
                    tests::{test_function_name, test_zome_api_function_wasm, test_zome_name},
                    ZomeApiFunction,
                },
                Defn,
            },
            state::tests::test_nucleus_state,
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
        json::{JsonString, RawString},
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

    /// dummy zome name compatible with ZomeFnCall
    pub fn test_zome() -> String {
        "test_zome".to_string()
    }

    /// dummy capability token
    pub fn test_capability_token() -> Address {
        Address::from(test_capability_token_str())
    }

    /// dummy capability token compatible with ZomeFnCall
    pub fn test_capability_token_str() -> String {
        "test_token".to_string()
    }

    /// test capability call
    pub fn test_capability_call<J: Into<JsonString>>(
        context: Arc<Context>,
        function: &str,
        parameters: J,
    ) -> CapabilityCall {
        make_cap_call(
            context.clone(),
            test_capability_token(),
            Address::from(context.agent_id.key.clone()),
            function,
            parameters,
        )
    }

    /// test self agent capability call
    pub fn test_agent_capability_call<J: Into<JsonString>>(
        context: Arc<Context>,
        function: &str,
        parameters: J,
    ) -> CapabilityCall {
        make_cap_call(
            context.clone(),
            Address::from(context.agent_id.key.clone()),
            Address::from(context.agent_id.key.clone()),
            function,
            parameters,
        )
    }
    /// dummy capability call
    pub fn dummy_capability_call() -> CapabilityCall {
        CapabilityCall::new(
            test_capability_token(),
            Address::from("test caller"),
            CallSignature::default(),
        )
    }

    /// dummy capability name compatible with ZomeFnCall
    pub fn test_capability_name() -> String {
        "test_cap".to_string()
    }

    /// dummy function name compatible with ZomeFnCall
    pub fn test_function() -> String {
        "test_function".to_string()
    }

    /// dummy parameters compatible with ZomeFnCall
    pub fn test_parameters() -> String {
        "".to_string()
    }

    /// dummy function call
    pub fn test_zome_call() -> ZomeFnCall {
        ZomeFnCall::new(
            &test_zome(),
            Some(dummy_capability_call()),
            &test_function(),
            test_parameters(),
        )
    }

    /// dummy function result
    pub fn test_call_response() -> ExecuteZomeFnResponse {
        ExecuteZomeFnResponse::new(test_zome_call(), Ok("foo".into()))
    }

    #[test]
    /// test the equality and uniqueness of function calls (based on internal snowflakes)
    fn test_zome_call_eq() {
        let zc1 = test_zome_call();
        let zc2 = test_zome_call();

        assert_eq!(zc1, zc1);
        assert_ne!(zc1, zc2);
    }

    #[test]
    /// test access to function result's function call
    fn test_zome_call_result() {
        let zome_call = test_zome_call();
        let call_result = ExecuteZomeFnResponse::new(zome_call.clone(), Ok("foo".into()));

        assert_eq!(call_result.call(), zome_call);
    }

    #[test]
    /// test access to the result of function result
    fn test_call_result_result() {
        assert_eq!(test_call_response().result(), Ok("foo".into()),);
    }

    #[test]
    /// tests that calling a valid zome function returns a valid result
    fn call_zome_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let (mut instance, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");

        // Create zome function call
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(test_capability_call(context, "public_test_fn", "")),
            "public_test_fn",
            "",
        );

        let result = super::call_and_wait_for_result(zome_call, &mut instance);

        assert!(result.is_ok());
        assert_eq!(JsonString::from(RawString::from(1337)), result.unwrap());
    }

    #[test]
    /// smoke test reducing over a nucleus
    fn can_reduce_execfn_action() {
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = sync_channel::<ActionWrapper>(10);
        let (tx_observer, _observer) = sync_channel::<Observer>(10);
        let context = test_context_with_channels("jimmy", &sender, &tx_observer, None);
        let call = ZomeFnCall::new(
            "myZome",
            Some(test_capability_call(context.clone(), "bogusfn", "")),
            "bogusfn",
            "",
        );

        let action_wrapper = ActionWrapper::new(Action::ExecuteZomeFunction(call));

        let reduced_nucleus = reduce(context, nucleus.clone(), &action_wrapper);
        assert_eq!(nucleus, reduced_nucleus);
    }

    #[test]
    /// tests that calling an invalid DNA returns the correct error
    fn call_ribosome_wrong_dna() {
        let netname = Some("call_ribosome_wrong_dna");
        let mut instance = Instance::new(test_context("janet", netname));

        instance.start_action_loop(test_context("jane", netname));

        let call = ZomeFnCall::new(
            "test_zome",
            Some(dummy_capability_call()),
            "public_test_fn",
            "{}",
        );
        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::DnaMissing) => {}
            _ => assert!(false),
        }
    }

    #[test]
    /// tests that calling a valid zome with invalid function returns the correct error
    fn call_ribosome_wrong_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let mut instance = test_instance(dna, None).expect("Could not initialize test instance");

        // Create zome function call:
        let call = ZomeFnCall::new("test_zome", Some(dummy_capability_call()), "xxx", "{}");

        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::Dna(DnaError::ZomeFunctionNotFound(err))) => {
                assert_eq!(err, "Zome function \'xxx\' not found in Zome 'test_zome'")
            }
            _ => assert!(false),
        }
    }

    #[test]
    /// tests that calling the wrong zome/capability returns the correct errors
    fn call_wrong_zome_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let mut instance = test_instance(dna, None).expect("Could not initialize test instance");

        // Create bad zome function call
        let call = ZomeFnCall::new("xxx", Some(dummy_capability_call()), "public_test_fn", "{}");

        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::Dna(err)) => assert_eq!(err.to_string(), "Zome 'xxx' not found"),
            _ => assert!(false),
        }

        /*
        convert when we actually have capabilities on a chain
                let mut cap_call = test_capability_call();
                cap_call.cap_name = "xxx".to_string();

                // Create bad capability function call
        let call = ZomeFnCall::new("test_zome", Some(cap_call), "public_test_fn", "{}");

                let result = super::call_and_wait_for_result(call, &mut instance);

                match result {
                    Err(HolochainError::Dna(err)) => assert_eq!(
                        err.to_string(),
                        "Capability 'xxx' not found in Zome 'test_zome'"
                    ),
                    _ => assert!(false),
                }
        */
    }

    #[test]
    fn test_zomefncall_same_as() {
        let base = ZomeFnCall::new("yoyo", Some(dummy_capability_call()), "fufu", "papa");
        let copy = ZomeFnCall::new("yoyo", Some(dummy_capability_call()), "fufu", "papa");
        let same = ZomeFnCall::new("yoyo", Some(dummy_capability_call()), "fufu", "papa1");
        let diff1 = ZomeFnCall::new("yoyo1", Some(dummy_capability_call()), "fufu", "papa");
        let diff2 = ZomeFnCall::new("yoyo", Some(dummy_capability_call()), "fufu3", "papa");

        assert_ne!(base, copy);
        assert!(base.same_fn_as(&copy));
        assert!(copy.same_fn_as(&base));
        assert!(base.same_fn_as(&same));
        assert!(!base.same_fn_as(&diff1));
        assert!(!base.same_fn_as(&diff2));
    }

    #[test]
    /// test for returning zome function result actions
    fn test_reduce_return_zome_function_result() {
        let context = test_context("jimmy", None);
        let mut state = test_nucleus_state();
        let action_wrapper = test_action_wrapper_rzfr();

        // @TODO don't juggle action wrappers to get at action in state
        // @see https://github.com/holochain/holochain-rust/issues/198
        let action = action_wrapper.action();
        let fr = unwrap_to!(action => Action::ReturnZomeFunctionResult);

        reduce_return_zome_function_result(context, &mut state, &action_wrapper);

        assert!(state.zome_calls.contains_key(&fr.call()));
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
            CallSignature::default(),
        );
        test_reduce_call(
            &test_setup,
            Some(cap_call.clone()),
            expected_failure.clone(),
        );

        // make the call with an valid capability call from self
        let cap_call = test_agent_capability_call(test_setup.context.clone(), "test", "{}");
        test_reduce_call(&test_setup, Some(cap_call), SUCCESS_EXPECTED.clone());

        // make the call with an invalid valid capability call from self
        let cap_call = test_agent_capability_call(test_setup.context.clone(), "some_fn", "{}");
        test_reduce_call(&test_setup, Some(cap_call), expected_failure);

        // make the call with an valid capability call from a different sources
        let grant = CapTokenGrant::create(CapabilityType::Transferable, None).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let addr = block_on(author_entry(&grant_entry, None, &test_setup.context)).unwrap();
        let cap_call = make_cap_call(
            test_setup.context.clone(),
            addr,
            Address::from("any caller"),
            "test",
            "{}",
        );
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
            CallSignature::default(),
        );
        test_reduce_call(&test_setup, Some(cap_call), expected_failure.clone());

        // test assigned capability where the caller is the agent
        let agent_token_str = test_setup.context.agent_id.key.clone();
        let cap_call = make_cap_call(
            test_setup.context.clone(),
            Address::from(agent_token_str.clone()),
            Address::from(agent_token_str),
            "test",
            "{}",
        );
        test_reduce_call(&test_setup, Some(cap_call), SUCCESS_EXPECTED.clone());

        // test assigned capability where the caller is someone else
        let someone = Address::from("somoeone");
        let grant =
            CapTokenGrant::create(CapabilityType::Assigned, Some(vec![someone.clone()])).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let grant_addr = block_on(author_entry(&grant_entry, None, &test_setup.context)).unwrap();
        let cap_call = make_cap_call(
            test_setup.context.clone(),
            grant_addr.clone(),
            Address::from("any caller"),
            "test",
            "{}",
        );
        test_reduce_call(&test_setup, Some(cap_call), expected_failure.clone());

        // test assigned capability where the caller is someone else
        let cap_call = make_cap_call(
            test_setup.context.clone(),
            grant_addr,
            someone.clone(),
            "test",
            "{}",
        );
        test_reduce_call(&test_setup, Some(cap_call), SUCCESS_EXPECTED.clone());
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
    fn test_make_cap_call() {
        let context = test_context("alice", None);
        let cap_call = make_cap_call(
            context.clone(),
            test_capability_token(),
            Address::from("caller"),
            "some_fn",
            "{}",
        );
        assert_eq!(cap_call.cap_token, test_capability_token());
        assert_eq!(cap_call.caller, Address::from("caller"));
        assert_eq!(cap_call.signature, make_call_sig(context, "some_fn", "{}"));
    }

    #[test]
    fn test_validate_call_public() {
        let dna = setup_dna_for_cap_test(CapabilityType::Public);
        let test_setup = setup_test(dna);
        let context = test_setup.context;
        let state = context.state().unwrap().nucleus();

        // non existent functions should fail
        let zome_call = ZomeFnCall::new("test_zome", None, "foo_func", "{}");
        let result = validate_call(context.clone(), &state, &zome_call);
        assert_eq!(
            result,
            Err(HolochainError::Dna(DnaError::ZomeFunctionNotFound(
                String::from("Zome function \'foo_func\' not found in Zome \'test_zome\'")
            )))
        );

        // non existent zomes should fial
        let zome_call = ZomeFnCall::new("foo_zome", None, "test", "{}");
        let result = validate_call(context.clone(), &state, &zome_call);
        assert_eq!(
            result,
            Err(HolochainError::Dna(DnaError::ZomeNotFound(String::from(
                "Zome \'foo_zome\' not found"
            ))))
        );
    }

    #[test]
    fn test_validate_call_by_agent() {
        let dna = setup_dna_for_cap_test(CapabilityType::Transferable);
        let test_setup = setup_test(dna);
        let context = test_setup.context;
        let state = context.state().unwrap().nucleus();

        // non public call should fail
        let zome_call = ZomeFnCall::new("test_zome", None, "test", "{}");
        let result = validate_call(context.clone(), &state, &zome_call);
        assert_eq!(result, Err(HolochainError::CapabilityCheckFailed));

        // if the agent doesn't correctly sign the call it should fail
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(make_cap_call(
                context.clone(),
                Address::from(context.agent_id.key.clone()),
                Address::from(context.agent_id.key.clone()),
                "foo_function", //<- not the function in the zome_call!
                "{}",
            )),
            "test",
            "{}",
        );

        let result = validate_call(context.clone(), &state, &zome_call);
        assert_eq!(result, Err(HolochainError::CapabilityCheckFailed));

        // should work with correctly signed cap_call
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(make_cap_call(
                context.clone(),
                Address::from(context.agent_id.key.clone()),
                Address::from(context.agent_id.key.clone()),
                "test",
                "{}",
            )),
            "test",
            "{}",
        );
        let result = validate_call(context.clone(), &state, &zome_call);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_as_token() {
        let context = test_context("alice", None);
        let agent_token = Address::from(context.agent_id.key.clone());
        let cap_call = make_cap_call(
            context.clone(),
            agent_token.clone(),
            agent_token.clone(),
            "test",
            "{}",
        );
        assert!(is_token_the_agent(context.clone(), &cap_call));

        // bogus token should fail
        let cap_call = CapabilityCall::new(
            Address::from("fake_token"),
            Address::from("someone"),
            CallSignature::default(),
        );
        assert!(!is_token_the_agent(context, &cap_call));
    }

    #[test]
    fn test_get_grant() {
        let dna = setup_dna_for_cap_test(CapabilityType::Transferable);
        let test_setup = setup_test(dna);
        let grant = CapTokenGrant::create(CapabilityType::Transferable, None).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant.clone());
        let grant_addr = block_on(author_entry(&grant_entry, None, &test_setup.context)).unwrap();
        let context = test_setup.context;
        let maybe_grant = get_grant(context.clone(), &grant_addr);
        assert_eq!(maybe_grant, Some(grant));
    }

    #[test]
    fn test_check_capability_transferable() {
        let dna = setup_dna_for_cap_test(CapabilityType::Transferable);
        let test_setup = setup_test(dna);
        let context = test_setup.context;

        // no cap_call should fail
        let zome_call = ZomeFnCall::new("test_zome", None, "test", "{}");
        assert!(!check_capability(context.clone(), &zome_call));

        // bogus cap_call should fail
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(CapabilityCall::new(
                Address::from("foo_token"),
                Address::from("some caller"),
                CallSignature::default(),
            )),
            "test",
            "{}",
        );
        assert!(!check_capability(context.clone(), &zome_call));

        // add the transferable grant and get the token which is the grant's address
        let grant = CapTokenGrant::create(CapabilityType::Transferable, None).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let grant_addr = block_on(author_entry(&grant_entry, None, &context)).unwrap();

        // make the call with a valid capability call from a random source should succeed
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(make_cap_call(
                context.clone(),
                grant_addr,
                Address::from("some_random_agent"),
                "test",
                "{}",
            )),
            "test",
            "{}",
        );
        assert!(check_capability(context.clone(), &zome_call));
    }

    #[test]
    fn test_verify_grant() {
        let context = test_context("alice", None);
        let test_address1 = Address::from("agent 1");
        let test_address2 = Address::from("some other identity");

        let zome_call_with_no_cap_call = ZomeFnCall::new("test_zome", None, "test", "{}");

        fn zome_call_valid(context: Arc<Context>, token: &Address, addr: &Address) -> ZomeFnCall {
            ZomeFnCall::new(
                "test_zome",
                Some(make_cap_call(
                    context.clone(),
                    token.clone(),
                    addr.clone(),
                    "test",
                    "{}",
                )),
                "test",
                "{}",
            )
        }

        let zome_call_from_addr1_bad_token = &ZomeFnCall::new(
            "test_zome",
            Some(make_cap_call(
                context.clone(),
                Address::from("bad token"),
                test_address1.clone(),
                "test",
                "{}",
            )),
            "test",
            "{}",
        );

        let grant = CapTokenGrant::create(CapabilityType::Public, None).unwrap();
        let token = grant.token();
        assert!(verify_grant(
            context.clone(),
            &grant,
            &zome_call_with_no_cap_call
        ));
        assert!(verify_grant(
            context.clone(),
            &grant,
            &zome_call_valid(context.clone(), &token, &test_address1)
        ));
        assert!(verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_token
        ));

        let grant = CapTokenGrant::create(CapabilityType::Transferable, None).unwrap();
        let token = grant.token();
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_with_no_cap_call
        ));
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_token
        ));

        // call with cap_call for a different function than the zome call
        let zome_call_from_addr1_bad_cap_call = &ZomeFnCall::new(
            "test_zome",
            Some(make_cap_call(
                context.clone(),
                token.clone(),
                test_address1.clone(),
                "foo-fn",
                "{}",
            )),
            "test",
            "{}",
        );
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_cap_call
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

        let grant =
            CapTokenGrant::create(CapabilityType::Assigned, Some(vec![test_address1.clone()]))
                .unwrap();
        let token = grant.token();
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_with_no_cap_call
        ));
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_token
        ));

        // call with cap_call for a different function than the zome call
        let zome_call_from_addr1_bad_cap_call = &ZomeFnCall::new(
            "test_zome",
            Some(make_cap_call(
                context.clone(),
                token.clone(),
                test_address1.clone(),
                "foo-fn",
                "{}",
            )),
            "test",
            "{}",
        );
        assert!(!verify_grant(
            context.clone(),
            &grant,
            &zome_call_from_addr1_bad_cap_call
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
