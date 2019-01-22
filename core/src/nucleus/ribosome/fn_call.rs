/// fn_call is the module that implements calling zome functions
///
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::{dispatch_action_with_observer, Observer},
    nucleus::{ribosome, state::NucleusState},
};
use holochain_core_types::{
    cas::content::Address,
    dna::{capabilities::CapabilityCall, wasm::DnaWasm},
    entry::cap_entries::CapTokenGrant,
    error::{HcResult, HolochainError,},
    json::JsonString,
};
use snowflake;
use std::{
    sync::mpsc::{sync_channel, SyncSender},
    convert::TryFrom, sync::Arc, thread
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
            reduce,
            ribosome::{
                api::{
                    tests::{
                        test_function_name, test_zome_api_function_wasm,
                        test_zome_name,
                    },
                    ZomeApiFunction,
                },
                Defn,
            },
            state::tests::test_nucleus_state,
        },
        action::tests::test_action_wrapper_rzfr,
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

    /// dummy capability call
    pub fn test_capability_call() -> CapabilityCall {
        CapabilityCall::new(
            test_capability_token(),
            Address::from("test caller"),
            CallSignature {},
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
            Some(test_capability_call()),
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
        let mut instance = test_instance(dna, None).expect("Could not initialize test instance");

        // Create zome function call
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(test_capability_call()),
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
        let call = ZomeFnCall::new("myZome", Some(test_capability_call()), "bogusfn", "");

        let action_wrapper = ActionWrapper::new(Action::ExecuteZomeFunction(call));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = sync_channel::<ActionWrapper>(10);
        let (tx_observer, _observer) = sync_channel::<Observer>(10);
        let context = test_context_with_channels("jimmy", &sender, &tx_observer, None);

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
            Some(test_capability_call()),
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
        let call = ZomeFnCall::new("test_zome", Some(test_capability_call()), "xxx", "{}");

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
        let call = ZomeFnCall::new("xxx", Some(test_capability_call()), "public_test_fn", "{}");

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
        let base = ZomeFnCall::new("yoyo", Some(test_capability_call()), "fufu", "papa");
        let copy = ZomeFnCall::new("yoyo", Some(test_capability_call()), "fufu", "papa");
        let same = ZomeFnCall::new("yoyo", Some(test_capability_call()), "fufu", "papa1");
        let diff1 = ZomeFnCall::new("yoyo1", Some(test_capability_call()), "fufu", "papa");
        let diff2 = ZomeFnCall::new("yoyo", Some(test_capability_call()), "fufu3", "papa");

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
