/// Nucleus is the module that handles DNA, including the Ribosome.
///
pub mod actions;
pub mod memory;
pub mod ribosome;
pub mod state;

use action::{Action, ActionWrapper, NucleusReduceFn};
use context::Context;
use error::HolochainError;
use holochain_dna::{wasm::DnaWasm, zome::capabilities::Capability, Dna, DnaError};
use instance::{dispatch_action_with_observer, Observer};
use nucleus::{
    ribosome::callback::{
        genesis::genesis, validate_commit::validate_commit, CallbackParams, CallbackResult,
    },
    state::{NucleusState, NucleusStatus},
};
use snowflake;
use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
};

use hash_table::sys_entry::ToEntry;
use nucleus::ribosome::api::call::reduce_call;

/// Struct holding data for requesting the execution of a Zome function (ExecutionZomeFunction Action)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZomeFnCall {
    id: snowflake::ProcessUniqueId,
    pub zome_name: String,
    pub cap_name: String,
    pub fn_name: String,
    pub parameters: String,
}

impl ZomeFnCall {
    pub fn new(zome: &str, capability: &str, function: &str, parameters: &str) -> Self {
        ZomeFnCall {
            // @TODO can we defer to the ActionWrapper id?
            // @see https://github.com/holochain/holochain-rust/issues/198
            id: snowflake::ProcessUniqueId::new(),
            zome_name: zome.to_string(),
            cap_name: capability.to_string(),
            fn_name: function.to_string(),
            parameters: parameters.to_string(),
        }
    }

    pub fn same_fn_as(&self, fn_call: &ZomeFnCall) -> bool {
        self.zome_name == fn_call.zome_name
            && self.cap_name == fn_call.cap_name
            && self.fn_name == fn_call.fn_name
    }
}

/// WIP - Struct for holding data when requesting an Entry Validation (ValidateEntry Action)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntrySubmission {
    pub zome_name: String,
    pub type_name: String,
    pub entry_content: String,
}

impl EntrySubmission {
    pub fn new<S: Into<String>>(zome_name: S, type_name: S, content: S) -> Self {
        EntrySubmission {
            zome_name: zome_name.into(),
            type_name: type_name.into(),
            entry_content: content.into(),
        }
    }
}

/// Dispatch ExecuteZoneFunction to and block until call has finished.
pub fn call_zome_and_wait_for_result(
    call: ZomeFnCall,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Result<String, HolochainError> {
    let call_action_wrapper = ActionWrapper::new(Action::ExecuteZomeFunction(call.clone()));

    // Dispatch action with observer closure that waits for a result in the state
    let (sender, receiver) = channel();
    dispatch_action_with_observer(
        action_channel,
        observer_channel,
        call_action_wrapper,
        move |state: &super::state::State| {
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
/// for test only??
pub fn call_and_wait_for_result(
    call: ZomeFnCall,
    instance: &mut super::instance::Instance,
) -> Result<String, HolochainError> {
    let call_action = ActionWrapper::new(Action::ExecuteZomeFunction(call.clone()));

    // Dispatch action with observer closure that waits for a result in the state
    let (sender, receiver) = channel();
    instance.dispatch_with_observer(call_action, move |state: &super::state::State| {
        if let Some(result) = state.nucleus().zome_call_result(&call) {
            sender
                .send(result.clone())
                .expect("local channel to be open");
            true
        } else {
            false
        }
    });

    // Block until we got that result through the channel:
    receiver.recv().expect("local channel to work")
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ZomeFnResult {
    call: ZomeFnCall,
    result: Result<String, HolochainError>,
}

impl ZomeFnResult {
    fn new(call: ZomeFnCall, result: Result<String, HolochainError>) -> Self {
        ZomeFnResult { call, result }
    }

    /// read only access to call
    pub fn call(&self) -> ZomeFnCall {
        self.call.clone()
    }

    /// read only access to result
    pub fn result(&self) -> Result<String, HolochainError> {
        self.result.clone()
    }
}

/// Reduce ReturnInitializationResult Action
/// On initialization success, set Initialized status
/// otherwise set the failed message
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_return_initialization_result(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    if state.status() != NucleusStatus::Initializing {
        state.status = NucleusStatus::InitializationFailed(
            "reduce of ReturnInitializationResult attempted when status != Initializing".into(),
        );
    } else {
        let action = action_wrapper.action();
        let result = unwrap_to!(action => Action::ReturnInitializationResult);
        match result {
            None => state.status = NucleusStatus::Initialized,
            Some(err) => state.status = NucleusStatus::InitializationFailed(err.clone()),
        };
    }
}

/// Helper
fn return_initialization_result(result: Option<String>, action_channel: &Sender<ActionWrapper>) {
    action_channel
        .send(ActionWrapper::new(Action::ReturnInitializationResult(
            result,
        )))
        .expect("action channel to be open in reducer");
}

/// Reduce InitApplication Action
/// Initialize the Holochain's Nucleus by doing the following:
/// Send Commit Action for Genesis Entry
/// Send ExecuteZomeFunction Action of the genesis callback of every zome
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_init_application(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) {
    // check pre-condition
    if state.status() != NucleusStatus::New {
        // Send bad start state ReturnInitializationResult Action
        return_initialization_result(
            Some("Nucleus already initialized or initializing".to_string()),
            &action_channel,
        );
        return;
    }

    let ia_action = action_wrapper.action();
    let dna = unwrap_to!(ia_action => Action::InitApplication);

    // Update status
    state.status = NucleusStatus::Initializing;

    // Set DNA
    state.dna = Some(dna.clone());

    // Create & launch thread
    let genesis_action_channel = action_channel.clone();
    let genesis_observer_channel = observer_channel.clone();
    let dna_clone = dna.clone();

    thread::spawn(move || {
        // Send Commit Action for Genesis Entry
        {
            // Create Commit Action for Genesis Entry
            let genesis_entry = dna_clone.to_entry();
            let commit_genesis_action = ActionWrapper::new(Action::Commit(genesis_entry));

            // Send Action and wait for it
            // TODO #249 - Do `dispatch_action_and_wait` instead to make sure dna commit succeeded
            ::instance::dispatch_action(&genesis_action_channel, commit_genesis_action);
        }

        // map genesis across every zome
        let mut results: Vec<_> = dna_clone
            .zomes
            .keys()
            .map(|zome_name| {
                genesis(
                    context.clone(),
                    &genesis_action_channel,
                    &genesis_observer_channel,
                    zome_name,
                    &CallbackParams::Genesis,
                )
            })
            .collect();

        // pad out a single pass if there are no zome results
        // @TODO #78 - is this really OK?
        // should we be steamrolling ahead with an instance that has no zomes and no
        // genesis?
        // actually this can cause some really nasty edge case bugs for code that assumes
        // there is a genesis (real example: wait for length 4 in history) in a loop before
        // moving forward. it will seem to work OK, but then hang if ever hit with an empty
        // Dna.
        // on the flip side, code cannot simply wait for 2 items in history, or even the
        // initialization result on its own, because then it will miss the genesis
        // sometimes where a genesis does happen.
        if results.is_empty() {
            results.push(CallbackResult::Pass);
        }

        // map the genesis results to initialization result responses
        for result in results {
            match result {
                CallbackResult::Fail(s) => {
                    return_initialization_result(Some(s.to_string()), &genesis_action_channel)
                }
                _ => return_initialization_result(None, &genesis_action_channel),
            }
        }
    });
}

pub(crate) fn launch_zome_fn_call(
    context: Arc<Context>,
    fc: ZomeFnCall,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    wasm: &DnaWasm,
    app_name: String,
) {
    let action_channel = action_channel.clone();
    let tx_observer = observer_channel.clone();
    let code = wasm.code.clone();
    thread::spawn(move || {
        let result: ZomeFnResult;
        match ribosome::api::call(
            &app_name,
            context,
            &action_channel,
            &tx_observer,
            code,
            &fc,
            Some(fc.clone().parameters.into_bytes()),
        ) {
            Ok(runtime) => {
                result = ZomeFnResult::new(fc.clone(), Ok(runtime.result.to_string()));
            }

            Err(ref error) => {
                result = ZomeFnResult::new(
                    fc.clone(),
                    Err(HolochainError::ErrorGeneric(format!("{}", error))),
                );
            }
        }
        // Send ReturnResult Action
        action_channel
            .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(result)))
            .expect("action channel to be open in reducer");
    });
}

/// Reduce ExecuteZomeFunction Action
/// Execute an exposed Zome function in a seperate thread and send the result in
/// a ReturnZomeFunctionResult Action on success or failure
fn reduce_execute_zome_function(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) {
    let fn_call = match action_wrapper.action().clone() {
        Action::ExecuteZomeFunction(call) => call,
        _ => unreachable!(),
    };

    fn dispatch_zome_fn_error(action_channel: &Sender<ActionWrapper>, fn_call: &ZomeFnCall, error: HolochainError) {
        let zome_not_found_result= ZomeFnResult::new(
            fn_call.clone(),
            Err(error.clone())
        );

        action_channel
            .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(zome_not_found_result)))
            .expect("action channel to be open in reducer");
    }

    let dna = match state.dna {
        Some(ref d) => d,
        None => {
            dispatch_zome_fn_error(action_channel, &fn_call, HolochainError::DnaMissing);
            return;
        }
    };


    // Walk through DNA and check if Zome, Capability, Function exists.
    // Create according errors if not.
    let wasm_for_function = match dna.zomes.get(&fn_call.zome_name) {
        None => Err(HolochainError::DnaError(DnaError::ZomeNotFound(
                format!("Zome '{}' not found", fn_call.zome_name.clone()),
            ))),
        Some(zome) => match zome.capabilities.get(&fn_call.cap_name) {
            None => Err(HolochainError::DnaError(DnaError::CapabilityNotFound(
                    format!("Capability '{}' not found in Zome '{}'", fn_call.cap_name.clone(), fn_call.zome_name.clone()),
                ))),

            Some(capability) => {
                match capability.functions.iter()
                    .find(|&fn_declaration| fn_declaration.name == fn_call.fn_name) {
                    None => Err(HolochainError::DnaError(DnaError::ZomeFunctionNotFound(
                                format!("Zome function '{}' not found", fn_call.fn_name.clone()),
                            ))),

                    Some(_) => {

                        // Ok Zome function is defined in given capability.
                        // Try getting this zome's WASM code:
                        match dna.get_wasm_from_zome_name(fn_call.zome_name.clone()) {
                            None => Err(HolochainError::DnaError(DnaError::ZomeFunctionNotFound(
                                format!("Zome '{}' has no binary code", fn_call.zome_name.clone()),
                            ))),

                            Some(wasm) => Ok(wasm)
                        }
                    },
                }
            },
        },
    };

    match wasm_for_function {
        Err(error) => dispatch_zome_fn_error(action_channel, &fn_call, error),
        Ok(wasm) => {
            // Prepare call - FIXME is this really useful?
            state.zome_calls.insert(fn_call.clone(), None);
            // Launch thread with function call
            launch_zome_fn_call(
                context,
                fn_call,
                action_channel,
                observer_channel,
                &wasm,
                state.dna.clone().unwrap().name,
            );
        }
    };
}

/// Reduce ValidateEntry Action
/// Validate an Entry by calling its validation function
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_validate_entry(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let entry = unwrap_to!(action => Action::ValidateEntry);
    match state
        .dna()
        .unwrap()
        .get_zome_name_for_entry_type(entry.entry_type())
    {
        None => {
            let error = format!("Unknown entry type: '{}'", entry.entry_type());
            state
                .validation_results
                .insert(action_wrapper.clone(), Err(error.to_string()));
        }
        Some(zome_name) => {
            #[cfg(debug)]
            state.validations_running.push(action_wrapper.clone());
            let action_channel = action_channel.clone();
            let observer_channel = observer_channel.clone();
            let action_wrapper = action_wrapper.clone();
            let entry = entry.clone();
            thread::spawn(move || {
                let validation_result = match validate_commit(
                    context.clone(),
                    &action_channel,
                    &observer_channel,
                    &zome_name,
                    &CallbackParams::ValidateCommit(entry.clone()),
                ) {
                    CallbackResult::Fail(error_string) => Err(error_string),
                    CallbackResult::Pass => Ok(()),
                    CallbackResult::NotImplemented => Err(format!(
                        "Validation callback not implemented for {:?}",
                        entry.entry_type()
                    )),
                };
                action_channel
                    .send(ActionWrapper::new(Action::ReturnValidationResult((
                        Box::new(action_wrapper),
                        validation_result,
                    ))))
                    .expect("action channel to be open in reducer");
            });
        }
    };
}
fn reduce_return_validation_result(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    _: &Sender<ActionWrapper>,
    _: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let (action_wrapper, validation_result) = unwrap_to!(action => Action::ReturnValidationResult);
    state
        .validation_results
        .insert(*action_wrapper.clone(), validation_result.clone());
    #[cfg(debug)]
    state
        .validations_running
        .retain(|x| x.id() != action_wrapper.id());
}

/// Reduce ReturnZomeFunctionResult Action.
/// Simply drops function call into zome_calls state.
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_return_zome_function_result(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let fr = unwrap_to!(action => Action::ReturnZomeFunctionResult);
    // @TODO store the action and result directly
    // @see https://github.com/holochain/holochain-rust/issues/198
    state.zome_calls.insert(fr.call(), Some(fr.result()));
}

/// Maps incoming action to the correct reducer
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NucleusReduceFn> {
    match action_wrapper.action() {
        Action::ReturnInitializationResult(_) => Some(reduce_return_initialization_result),
        Action::InitApplication(_) => Some(reduce_init_application),
        Action::ExecuteZomeFunction(_) => Some(reduce_execute_zome_function),
        Action::ReturnZomeFunctionResult(_) => Some(reduce_return_zome_function_result),
        Action::ValidateEntry(_) => Some(reduce_validate_entry),
        Action::Call(_) => Some(reduce_call),
        Action::ReturnValidationResult(_) => Some(reduce_return_validation_result),
        _ => None,
    }
}

/// Reduce state of Nucleus according to action.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<NucleusState>,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Arc<NucleusState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: NucleusState = (*old_state).clone();
            f(
                context,
                &mut new_state,
                &action_wrapper,
                action_channel,
                observer_channel,
            );
            Arc::new(new_state)
        }
        None => old_state,
    }
}

// Helper function for getting a Capability for a ZomeFnCall request
fn get_capability_with_zome_call(
    dna: &Dna,
    zome_call: &ZomeFnCall,
) -> Result<Capability, ZomeFnResult> {
    // Get Capability from DNA
    let res = dna.get_capability_with_zome_name(&zome_call.zome_name, &zome_call.cap_name);
    match res {
        Err(e) => Err(ZomeFnResult::new(
            zome_call.clone(),
            Err(HolochainError::DnaError(e)),
        )),
        Ok(cap) => Ok(cap.clone()),
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    use super::*;
    use action::{tests::test_action_wrapper_rzfr, ActionWrapper};
    use holochain_dna::Dna;
    use instance::{
        tests::{test_context, test_instance, test_instance_blank},
        Instance,
    };
    use nucleus::state::tests::test_nucleus_state;
    use std::sync::{mpsc::channel, Arc};

    use std::error::Error;

    /// dummy zome name compatible with ZomeFnCall
    pub fn test_zome() -> String {
        "foo zome".to_string()
    }

    /// dummy capability compatible with ZomeFnCall
    pub fn test_capability() -> String {
        "foo capability".to_string()
    }

    /// dummy function name compatible with ZomeFnCall
    pub fn test_function() -> String {
        "foo_function".to_string()
    }

    /// dummy parameters compatible with ZomeFnCall
    pub fn test_parameters() -> String {
        "".to_string()
    }

    /// dummy function call
    pub fn test_zome_call() -> ZomeFnCall {
        ZomeFnCall::new(
            &test_zome(),
            &test_capability(),
            &test_function(),
            &test_parameters(),
        )
    }

    /// dummy function result
    pub fn test_call_result() -> ZomeFnResult {
        ZomeFnResult::new(test_zome_call(), Ok("foo".to_string()))
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
        let call_result = ZomeFnResult::new(zome_call.clone(), Ok("foo".to_string()));

        assert_eq!(call_result.call(), zome_call);
    }

    #[test]
    /// test access to the result of function result
    fn test_call_result_result() {
        assert_eq!(test_call_result().result(), Ok("foo".to_string()));
    }

    #[test]
    /// smoke test the init of a nucleus
    fn can_instantiate_nucleus_state() {
        let nucleus_state = NucleusState::new();
        assert_eq!(nucleus_state.dna, None);
        assert_eq!(nucleus_state.has_initialized(), false);
        assert_eq!(nucleus_state.has_initialization_failed(), false);
        assert_eq!(nucleus_state.status(), NucleusStatus::New);
    }

    #[test]
    /// test for returning zome function result actions
    fn test_reduce_return_zome_function_result() {
        let context = test_context("jimmy");
        let instance = test_instance_blank();
        let mut state = test_nucleus_state();
        let action_wrapper = test_action_wrapper_rzfr();

        // @TODO don't juggle action wrappers to get at action in state
        // @see https://github.com/holochain/holochain-rust/issues/198
        let action = action_wrapper.action();
        let fr = unwrap_to!(action => Action::ReturnZomeFunctionResult);

        reduce_return_zome_function_result(
            context,
            &mut state,
            &action_wrapper,
            &instance.action_channel(),
            &instance.observer_channel(),
        );

        assert!(state.zome_calls.contains_key(&fr.call()));
    }

    #[test]
    /// smoke test the init of a nucleus reduction
    fn can_reduce_initialize_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(Action::InitApplication(dna));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, receiver) = channel::<ActionWrapper>();
        let (tx_observer, _observer) = channel::<Observer>();

        // Reduce Init action and block until receiving ReturnInit Action
        let reduced_nucleus = reduce(
            test_context("jimmy"),
            nucleus.clone(),
            &action_wrapper,
            &sender.clone(),
            &tx_observer.clone(),
        );
        receiver.recv().expect("channel failed");

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.has_initialization_failed(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);
    }

    #[test]
    /// test that we can initialize and send/receive result values from a nucleus
    fn can_reduce_return_init_result_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(Action::InitApplication(dna));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, receiver) = channel::<ActionWrapper>();
        let (tx_observer, _observer) = channel::<Observer>();

        // Reduce Init action and block until receiving ReturnInit Action
        let initializing_nucleus = reduce(
            test_context("jimmy"),
            nucleus.clone(),
            &action_wrapper,
            &sender.clone(),
            &tx_observer.clone(),
        );
        receiver.recv().expect("receiver fail");

        assert_eq!(initializing_nucleus.has_initialized(), false);
        assert_eq!(initializing_nucleus.has_initialization_failed(), false);
        assert_eq!(initializing_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(false) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(Action::ReturnInitializationResult(Some(
            "init failed".to_string(),
        )));
        let reduced_nucleus = reduce(
            test_context("jimmy"),
            initializing_nucleus.clone(),
            &return_action_wrapper,
            &sender.clone(),
            &tx_observer.clone(),
        );

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.has_initialization_failed(), true);
        assert_eq!(
            reduced_nucleus.status(),
            NucleusStatus::InitializationFailed("init failed".to_string())
        );

        // Reduce Init action and block until receiving ReturnInit Action
        let reduced_nucleus = reduce(
            test_context("jane"),
            reduced_nucleus.clone(),
            &action_wrapper,
            &sender.clone(),
            &tx_observer.clone(),
        );
        receiver.recv().expect("receiver shouldn't fail");

        assert_eq!(
            reduced_nucleus.status(),
            NucleusStatus::InitializationFailed("init failed".to_string())
        );

        // Send ReturnInit(None) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(Action::ReturnInitializationResult(None));
        let reduced_nucleus = reduce(
            test_context("jimmy"),
            initializing_nucleus.clone(),
            &return_action_wrapper,
            &sender.clone(),
            &tx_observer.clone(),
        );

        assert_eq!(reduced_nucleus.has_initialized(), true);
        assert_eq!(reduced_nucleus.has_initialization_failed(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initialized);
    }

    #[test]
    /// smoke test reducing over a nucleus
    fn can_reduce_execfn_action() {
        let call = ZomeFnCall::new("myZome", "public", "bogusfn", "");

        let action_wrapper = ActionWrapper::new(Action::ExecuteZomeFunction(call));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = channel::<ActionWrapper>();
        let (tx_observer, _observer) = channel::<Observer>();
        let reduced_nucleus = reduce(
            test_context("jimmy"),
            nucleus.clone(),
            &action_wrapper,
            &sender,
            &tx_observer,
        );
        assert_eq!(nucleus, reduced_nucleus);
    }

    #[test]
    /// tests that calling a valid zome function returns a valid result
    fn call_zome_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let mut instance = test_instance(dna);

        // Create zome function call
        let zome_call = ZomeFnCall::new("test_zome", "test_cap", "main", "");

        let result = super::call_and_wait_for_result(zome_call, &mut instance);
        match result {
            // Result 1337 from WASM (as string)
            Ok(val) => assert_eq!(val, "1337"),
            Err(err) => assert_eq!(err, HolochainError::InstanceActive),
        }
    }

    #[test]
    /// tests that calling an invalid DNA returns the correct error
    fn call_ribosome_wrong_dna() {
        let mut instance = Instance::new();

        instance.start_action_loop(test_context("jane"));

        let call = ZomeFnCall::new("test_zome", "test_cap", "main", "{}");
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
        let mut instance = test_instance(dna);

        // Create zome function call:
        let call = ZomeFnCall::new("test_zome", "test_cap", "xxx", "{}");

        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::ErrorGeneric(err)) => {
                assert_eq!(err, "Function: Module doesn\'t have export xxx")
            }
            _ => assert!(false),
        }
    }

    #[test]
    /// tests that calling the wrong zome/capability returns the correct errors
    fn call_wrong_zome_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let mut instance = test_instance(dna);

        // Create bad zome function call
        let call = ZomeFnCall::new("xxx", "test_cap", "main", "{}");

        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::DnaError(err)) => {
                assert_eq!(err.description(), "Zome 'xxx' not found")
            }
            _ => assert!(false),
        }

        // Create bad capability function call
        let call = ZomeFnCall::new("test_zome", "xxx", "main", "{}");

        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::DnaError(err)) => assert_eq!(
                err.description(),
                "Capability 'xxx' not found in Zome 'test_zome'"
            ),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_zomefncall_same_as() {
        let base = ZomeFnCall::new("zozo", "caca", "fufu", "papa");
        let copy = ZomeFnCall::new("zozo", "caca", "fufu", "papa");
        let same = ZomeFnCall::new("zozo", "caca", "fufu", "papa1");
        let diff1 = ZomeFnCall::new("zozo1", "caca", "fufu", "papa");
        let diff2 = ZomeFnCall::new("zozo", "caca2", "fufu", "papa");
        let diff3 = ZomeFnCall::new("zozo", "caca", "fufu3", "papa");

        assert_ne!(base, copy);
        assert!(base.same_fn_as(&copy));
        assert!(copy.same_fn_as(&base));
        assert!(base.same_fn_as(&same));
        assert!(!base.same_fn_as(&diff1));
        assert!(!base.same_fn_as(&diff2));
        assert!(!base.same_fn_as(&diff3));
    }
}
