pub mod memory;
pub mod ribosome;
pub mod state;

use context::Context;
use error::HolochainError;

use action::{Action, ActionWrapper, NucleusReduceFn};
use instance::Observer;
use nucleus::{
    ribosome::callback::{genesis::genesis, CallbackParams, CallbackResult},
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

/// Struct holding data for requesting the execution of a Zome function (ExecutionZomeFunction Action)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionCall {
    id: snowflake::ProcessUniqueId,
    pub zome: String,
    pub capability: String,
    pub function: String,
    pub parameters: String,
}

impl FunctionCall {
    pub fn new(zome: &str, capability: &str, function: &str, parameters: &str) -> Self {
        FunctionCall {
            id: snowflake::ProcessUniqueId::new(),
            zome: zome.to_string(),
            capability: capability.to_string(),
            function: function.to_string(),
            parameters: parameters.to_string(),
        }
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
    call: FunctionCall,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Result<String, HolochainError> {
    let call_action_wrapper = ActionWrapper::new(&Action::ExecuteZomeFunction(call.clone()));

    // Dispatch action with observer closure that waits for a result in the state
    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        action_channel,
        observer_channel,
        &call_action_wrapper,
        move |state: &super::state::State| {
            if let Some(result) = state.nucleus().ribosome_call_result(&call) {
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
    call: FunctionCall,
    instance: &mut super::instance::Instance,
) -> Result<String, HolochainError> {
    let call_action = ActionWrapper::new(&Action::ExecuteZomeFunction(call.clone()));

    // Dispatch action with observer closure that waits for a result in the state
    let (sender, receiver) = channel();
    instance.dispatch_with_observer(&call_action, move |state: &super::state::State| {
        if let Some(result) = state.nucleus().ribosome_call_result(&call) {
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
pub struct FunctionResult {
    call: FunctionCall,
    result: Result<String, HolochainError>,
}

impl FunctionResult {
    fn new(call: FunctionCall, result: Result<String, HolochainError>) -> Self {
        FunctionResult { call, result }
    }

    /// read only access to call
    pub fn call(&self) -> FunctionCall {
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
fn reduce_rir(
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
        .send(ActionWrapper::new(&Action::ReturnInitializationResult(
            result,
        )))
        .expect("action channel to be open in reducer");
}

/// Reduce InitApplication Action
/// Initialize Nucleus by setting the DNA
/// and sending ExecuteFunction Action of genesis of each zome
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_ia(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) {
    match state.status() {
        NucleusStatus::New => {
            let action = action_wrapper.action();
            let dna = unwrap_to!(action => Action::InitApplication);

            // Update status
            state.status = NucleusStatus::Initializing;

            // Set DNA
            state.dna = Some(dna.clone());

            // Create & launch thread
            let genesis_action_channel = action_channel.clone();
            let genesis_observer_channel = observer_channel.clone();
            let dna_clone = dna.clone();

            thread::spawn(move || {
                // map genesis across every zome
                let mut results: Vec<_> = dna_clone
                    .zomes
                    .iter()
                    .map(|zome| {
                        genesis(
                            &genesis_action_channel,
                            &genesis_observer_channel,
                            &zome.name(),
                            &CallbackParams::Genesis,
                        )
                    })
                    .collect();

                // pad out a single pass if there are no zome results
                // @TODO is this really OK?
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
                        CallbackResult::Fail(s) => return_initialization_result(
                            Some(s.to_string()),
                            &genesis_action_channel,
                        ),
                        _ => return_initialization_result(None, &genesis_action_channel),
                    }
                }
            });
        }
        _ => {
            // Send bad start state ReturnInitializationResult Action
            return_initialization_result(
                Some("Nucleus already initialized or initializing".to_string()),
                &action_channel,
            );
        }
    }
}

/// Reduce ExecuteZomeFunction Action
/// Execute an exposed Zome function in a seperate thread and send the result in
/// a ReturnZomeFunctionResult Action on success or failure
fn reduce_ezf(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) {
    let function_call = match action_wrapper.action() {
        Action::ExecuteZomeFunction(call) => call,
        _ => unreachable!(),
    };
    let fc = function_call.clone();

    let mut has_error = false;
    let mut result = FunctionResult::new(
        fc.clone(),
        Err(HolochainError::ErrorGeneric("[]".to_string())),
    );

    if let Some(ref dna) = state.dna {
        if let Some(ref zome) = dna.get_zome(&fc.zome) {
            if let Some(ref wasm) = dna.get_capability(zome, &fc.capability) {
                state.ribosome_calls.insert(fc.clone(), None);

                let action_channel = action_channel.clone();
                let tx_observer = observer_channel.clone();
                let code = wasm.code.clone();

                thread::spawn(move || {
                    let result: FunctionResult;
                    match ribosome::api::call(
                        context,
                        &action_channel,
                        &tx_observer,
                        code,
                        &function_call,
                        Some(function_call.clone().parameters.into_bytes()),
                    ) {
                        Ok(runtime) => {
                            result = FunctionResult::new(
                                function_call.clone(),
                                Ok(runtime.result.to_string()),
                            );
                        }

                        Err(ref error) => {
                            result = FunctionResult::new(
                                function_call.clone(),
                                Err(HolochainError::ErrorGeneric(format!("{}", error))),
                            );
                        }
                    }

                    // Send ReturnResult Action
                    action_channel
                        .send(ActionWrapper::new(&Action::ReturnZomeFunctionResult(
                            result,
                        )))
                        .expect("action channel to be open in reducer");
                });
            } else {
                has_error = true;
                result = FunctionResult::new(
                    fc.clone(),
                    Err(HolochainError::CapabilityNotFound(format!(
                        "Capability '{:?}' not found in Zome '{:?}'",
                        &fc.capability, &fc.zome
                    ))),
                );
            }
        } else {
            has_error = true;
            result = FunctionResult::new(
                fc.clone(),
                Err(HolochainError::ZomeNotFound(format!(
                    "Zome '{:?}' not found",
                    &fc.zome
                ))),
            );
        }
    } else {
        has_error = true;
        result = FunctionResult::new(fc.clone(), Err(HolochainError::DnaMissing));
    }
    if has_error {
        action_channel
            .send(ActionWrapper::new(&Action::ReturnZomeFunctionResult(
                result,
            )))
            .expect("action channel to be open in reducer");
    }
}

/// Reduce ValidateEntry Action
/// Validate an Entry by calling its validation function
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_ve(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let mut _has_entry_type = false;

    // must have entry_type
    if let Some(ref dna) = state.dna {
        let action = action_wrapper.action();
        let es = unwrap_to!(action => Action::ValidateEntry);
        if let Some(ref _wasm) =
            dna.get_validation_bytecode_for_entry_type(&es.zome_name, &es.type_name)
        {
            // TODO #61 validate()
            // Do same thing as Action::ExecuteZomeFunction
            _has_entry_type = true;
        }
    }
}

/// reduce ReturnZomeFunctionResult
/// simply drops function call into ribosome_calls state
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_rzfr(
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
    state.ribosome_calls.insert(fr.call(), Some(fr.result()));
}

fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NucleusReduceFn> {
    match action_wrapper.action() {
        Action::ReturnInitializationResult(_) => Some(reduce_rir),
        Action::InitApplication(_) => Some(reduce_ia),
        Action::ExecuteZomeFunction(_) => Some(reduce_ezf),
        Action::ReturnZomeFunctionResult(_) => Some(reduce_rzfr),
        Action::ValidateEntry(_) => Some(reduce_ve),
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

    /// dummy zome name compatible with FunctionCall
    pub fn test_zome() -> String {
        "foo zome".to_string()
    }

    /// dummy capability compatible with FunctionCall
    pub fn test_capability() -> String {
        "foo capability".to_string()
    }

    /// dummy function name compatible with FunctionCall
    pub fn test_function() -> String {
        "foo_function".to_string()
    }

    /// dummy parameters compatible with FunctionCall
    pub fn test_parameters() -> String {
        "".to_string()
    }

    /// dummy function call
    pub fn test_function_call() -> FunctionCall {
        FunctionCall::new(
            &test_zome(),
            &test_capability(),
            &test_function(),
            &test_parameters(),
        )
    }

    /// dummy function result
    pub fn test_function_result() -> FunctionResult {
        FunctionResult::new(test_function_call(), Ok("foo".to_string()))
    }

    #[test]
    /// test the equality and uniqueness of function calls (based on internal snowflakes)
    fn test_function_call_eq() {
        let fc1 = test_function_call();
        let fc2 = test_function_call();

        assert_eq!(fc1, fc1);
        assert_ne!(fc1, fc2);
    }

    #[test]
    /// test access to function result's function call
    fn test_function_result_call() {
        let fc = test_function_call();
        let fr = FunctionResult::new(fc.clone(), Ok("foo".to_string()));

        assert_eq!(fr.call(), fc);
    }

    #[test]
    /// test access to the result of function result
    fn test_function_result_result() {
        assert_eq!(test_function_result().result(), Ok("foo".to_string()));
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
    fn test_reduce_rzfr() {
        let context = test_context("jimmy");
        let instance = test_instance_blank();
        let mut state = test_nucleus_state();
        let action_wrapper = test_action_wrapper_rzfr();

        // @TODO don't juggle action wrappers to get at action in state
        // @see https://github.com/holochain/holochain-rust/issues/198
        let action = action_wrapper.action();
        let fr = unwrap_to!(action => Action::ReturnZomeFunctionResult);

        reduce_rzfr(
            context,
            &mut state,
            &action_wrapper,
            &instance.action_channel(),
            &instance.observer_channel(),
        );

        assert!(state.ribosome_calls.contains_key(&fr.call()));
    }

    #[test]
    /// smoke test the init of a nucleus reduction
    fn can_reduce_initialize_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(&Action::InitApplication(dna));
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
        receiver.recv().unwrap_or_else(|_| panic!("channel failed"));

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.has_initialization_failed(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);
    }

    #[test]
    /// test that we can initialize and send/receive result values from a nucleus
    fn can_reduce_return_init_result_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(&Action::InitApplication(dna));
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
        receiver.recv().unwrap_or_else(|_| panic!("receiver fail"));

        assert_eq!(initializing_nucleus.has_initialized(), false);
        assert_eq!(initializing_nucleus.has_initialization_failed(), false);
        assert_eq!(initializing_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(false) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(&Action::ReturnInitializationResult(Some(
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
        receiver.recv().unwrap_or_else(|_| panic!("receiver fail"));

        assert_eq!(
            reduced_nucleus.status(),
            NucleusStatus::InitializationFailed("init failed".to_string())
        );

        // Send ReturnInit(None) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(&Action::ReturnInitializationResult(None));
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
        let call = FunctionCall::new("myZome", "public", "bogusfn", "");

        let action_wrapper = ActionWrapper::new(&Action::ExecuteZomeFunction(call));
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
    fn call_ribosome_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let mut instance = test_instance(dna);

        // Create zome function call
        let call = FunctionCall::new("test_zome", "test_cap", "main", "");

        let result = super::call_and_wait_for_result(call, &mut instance);
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

        let call = FunctionCall::new("test_zome", "test_cap", "main", "{}");
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
        let call = FunctionCall::new("test_zome", "test_cap", "xxx", "{}");

        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::ErrorGeneric(err)) => {
                assert_eq!(err, "Function: Module doesn\'t have export xxx_dispatch")
            }
            _ => assert!(false),
        }
    }

    #[test]
    /// tests that calling the wrong zome/capability returns the correct errors
    fn call_wrong_ribosome_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let mut instance = test_instance(dna);

        // Create bad zome function call
        let call = FunctionCall::new("xxx", "test_cap", "main", "{}");

        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::ZomeNotFound(err)) => assert_eq!(err, "Zome '\"xxx\"' not found"),
            _ => assert!(false),
        }

        // Create bad capability function call
        let call = FunctionCall::new("test_zome", "xxx", "main", "{}");

        let result = super::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::CapabilityNotFound(err)) => assert_eq!(
                err,
                "Capability '\"xxx\"' not found in Zome '\"test_zome\"'"
            ),
            _ => assert!(false),
        }
    }

}
