pub mod ribosome;

use error::HolochainError;
use holochain_dna::{
    zome::capabilities::{ReservedCapabilityNames, ReservedFunctionNames}, Dna,
};
use instance::Observer;
use snowflake;
use state;
use std::{
    collections::HashMap, sync::{
        mpsc::{channel, Sender}, Arc,
    }, thread,
};

#[derive(Clone, Debug, PartialEq)]
pub enum NucleusStatus {
    New,
    Initializing,
    Initialized,
}

impl Default for NucleusStatus {
    fn default() -> Self {
        NucleusStatus::New
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NucleusState {
    dna: Option<Dna>,
    status: NucleusStatus,
    ribosome_calls: HashMap<FunctionCall, Option<Result<String, HolochainError>>>,
}

impl NucleusState {
    pub fn new() -> Self {
        NucleusState {
            dna: None,
            status: NucleusStatus::New,
            ribosome_calls: HashMap::new(),
        }
    }

    pub fn ribosome_call_result(
        &self,
        function_call: &FunctionCall,
    ) -> Option<Result<String, HolochainError>> {
        match self.ribosome_calls.get(function_call) {
            None => None,
            Some(value) => value.clone(),
        }
    }

    pub fn has_initialized(&self) -> bool {
        self.status == NucleusStatus::Initialized
    }

    // Getters
    pub fn dna(&self) -> Option<Dna> {
        self.dna.clone()
    }
    pub fn status(&self) -> NucleusStatus {
        self.status.clone()
    }
}

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
    pub fn new<S: Into<String>>(zome: S, capability: S, function: S, parameters: S) -> Self {
        FunctionCall {
            id: snowflake::ProcessUniqueId::new(),
            zome: zome.into(),
            capability: capability.into(),
            function: function.into(),
            parameters: parameters.into(),
        }
    }
}

/// WIP - Struct for holding data when requesting an Entry Validation (ValidateEntry Action)
#[derive(Clone, Debug, PartialEq, Eq)]
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
    action_channel: &Sender<::state::ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Result<String, HolochainError> {
    let call_action = super::state::Action::Nucleus(Action::ExecuteZomeFunction(call.clone()));

    // Dispatch action with observer closure that waits for a result in the state
    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        action_channel,
        observer_channel,
        call_action,
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
    let call_action = super::state::Action::Nucleus(Action::ExecuteZomeFunction(call.clone()));

    // Dispatch action with observer closure that waits for a result in the state
    let (sender, receiver) = channel();
    instance.dispatch_with_observer(call_action, move |state: &super::state::State| {
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

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionResult {
    call: FunctionCall,
    result: Result<String, HolochainError>,
}

impl FunctionResult {
    fn new(call: FunctionCall, result: Result<String, HolochainError>) -> Self {
        FunctionResult { call, result }
    }
}

/// Enum of all Actions that mutates the Nucleus's state
#[derive(Clone, Debug, PartialEq)]
#[allow(unknown_lints)]
#[allow(large_enum_variant)]
pub enum Action {
    InitApplication(Dna),
    ReturnInitializationResult(bool),
    ExecuteZomeFunction(FunctionCall),
    ReturnZomeFunctionResult(FunctionResult),
    ValidateEntry(EntrySubmission),
}

/// Reduce ReturnInitializationResult Action
/// On initialization success, set Initialized status
/// otherwise reset the nucleus
fn reduce_rir(nucleus_state: &mut NucleusState, has_succeeded: bool) {
    assert!(nucleus_state.status == NucleusStatus::Initializing);
    if has_succeeded {
        (*nucleus_state).status = NucleusStatus::Initialized
    } else {
        *nucleus_state = NucleusState::new();
    };
}

/// Reduce state of Nucleus according to action.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
pub fn reduce(
    old_state: Arc<NucleusState>,
    action: &state::Action,
    action_channel: &Sender<state::ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Arc<NucleusState> {
    match *action {
        state::Action::Nucleus(ref nucleus_action) => {
            let mut new_nucleus_state: NucleusState = (*old_state).clone();

            match *nucleus_action {
                Action::ReturnInitializationResult(has_succeeded) => {
                    reduce_rir(&mut new_nucleus_state, has_succeeded);
                }

                // Initialize Nucleus by setting the DNA
                // and sending ExecuteFunction Action of genesis of each zome
                Action::InitApplication(ref dna) => {
                    match new_nucleus_state.status {
                        NucleusStatus::New => {
                            // Update status
                            new_nucleus_state.status = NucleusStatus::Initializing;

                            // Set DNA
                            new_nucleus_state.dna = Some(dna.clone());

                            // Create & launch thread
                            let action_channel = action_channel.clone();
                            let observer_channel = observer_channel.clone();
                            let dna_clone = dna.clone();

                            thread::spawn(move || {
                                //  Call each Zome's genesis() with an ExecuteZomeFunction Action
                                for zome in dna_clone.zomes {
                                    // Make ExecuteZomeFunction Action for genesis()
                                    let call = FunctionCall::new(
                                        zome.name,
                                        ReservedCapabilityNames::LifeCycle.as_str().to_string(),
                                        ReservedFunctionNames::Genesis.as_str().to_string(),
                                        "".to_string(),
                                    );

                                    // Call Genesis and wait
                                    let call_result = call_zome_and_wait_for_result(
                                        call,
                                        &action_channel,
                                        &observer_channel,
                                    );

                                    // genesis returns a i32 as a string
                                    // 0 == success
                                    match call_result {
                                        // not okay if genesis returned an errorCode
                                        Ok(ref s) if s != "0" => {
                                            // Send a failed ReturnInitializationResult Action
                                            action_channel
                                              .send(state::ActionWrapper::new(state::Action::Nucleus(
                                                  Action::ReturnInitializationResult(false),
                                              )))
                                              .expect("action channel to be open in reducer");
                                            // Kill thread
                                            // TODO - Instead, Keep track of each zome's initialization.
                                            // @see https://github.com/holochain/holochain-rust/issues/78
                                            // Mark this one as failed and continue with other zomes
                                            return;
                                        }
                                        // its okay if hc_lifecycle or genesis not present
                                        Ok(_) | Err(HolochainError::CapabilityNotFound(_)) => { /* NA */ }
                                        Err(HolochainError::ErrorGeneric(ref msg)) if msg == "Function: Module doesn\'t have export genesis_dispatch"
                                          => { /* NA */ }
                                        // Init fails if something failed in genesis called
                                        Err(_e) => {
                                            // TODO - Create test for this edge case
                                            // @see https://github.com/holochain/holochain-rust/issues/78
                                            // Send a failed ReturnInitializationResult Action
                                            action_channel
                                              .send(state::ActionWrapper::new(state::Action::Nucleus(
                                                  Action::ReturnInitializationResult(false),
                                              )))
                                              .expect("action channel to be open in reducer");
                                            // Kill thread
                                            // TODO - Instead, Keep track of each zome's initialization.
                                            // @see https://github.com/holochain/holochain-rust/issues/78
                                            // Mark this one as failed and continue with other zomes
                                            return;
                                        }
                                    }
                                }
                                // Send Succeeded ReturnInitializationResult Action
                                action_channel
                                    .send(state::ActionWrapper::new(state::Action::Nucleus(
                                        Action::ReturnInitializationResult(true),
                                    )))
                                    .expect("action channel to be open in reducer");
                            });
                        }
                        _ => {
                            // TODO better error reporting based on current state and logger
                            // https://github.com/holochain/holochain-rust/issues/21
                            println!("\t!! Nucleus already initialized or initializing");
                        }
                    }
                }

                // Execute an exposed Zome function in a seperate thread and send the result in
                // a ReturnZomeFunctionResult Action on success or failure
                Action::ExecuteZomeFunction(ref fc) => {
                    let function_call = fc.clone();
                    let mut has_error = false;
                    let mut result = FunctionResult::new(
                        fc.clone(),
                        Err(HolochainError::ErrorGeneric("[]".to_string())),
                    );

                    if let Some(ref dna) = new_nucleus_state.dna {
                        if let Some(ref zome) = dna.get_zome(&fc.zome) {
                            if let Some(ref wasm) = dna.get_capability(zome, &fc.capability) {
                                new_nucleus_state.ribosome_calls.insert(fc.clone(), None);

                                let action_channel = action_channel.clone();
                                let tx_observer = observer_channel.clone();
                                let code = wasm.code.clone();

                                thread::spawn(move || {
                                    let result: FunctionResult;
                                    match ribosome::call(
                                        &action_channel,
                                        &tx_observer,
                                        code,
                                        &function_call.function.clone(),
                                        Some(function_call.clone().parameters.into_bytes()),
                                    ) {
                                        Ok(runtime) => {
                                            result = FunctionResult::new(
                                                function_call,
                                                Ok(runtime.result.to_string()),
                                            );
                                        }

                                        Err(ref error) => {
                                            result = FunctionResult::new(
                                                function_call,
                                                Err(HolochainError::ErrorGeneric(format!(
                                                    "{}",
                                                    error
                                                ))),
                                            );
                                        }
                                    }

                                    // Send ReturnResult Action
                                    action_channel
                                        .send(state::ActionWrapper::new(state::Action::Nucleus(
                                            Action::ReturnZomeFunctionResult(result),
                                        )))
                                        .expect("action channel to be open in reducer");
                                });
                            } else {
                                has_error = true;
                                result = FunctionResult::new(
                                    fc.clone(),
                                    Err(HolochainError::CapabilityNotFound(format!(
                                        "Capability '{}' not found in Zome '{}'",
                                        &fc.capability, &fc.zome
                                    ))),
                                );
                            }
                        } else {
                            has_error = true;
                            result = FunctionResult::new(
                                fc.clone(),
                                Err(HolochainError::ZomeNotFound(format!(
                                    "Zome '{}' not found",
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
                            .send(state::ActionWrapper::new(state::Action::Nucleus(
                                Action::ReturnZomeFunctionResult(result),
                            )))
                            .expect("action channel to be open in reducer");
                    }
                }

                // Store the Result in the ribosome_calls hashmap
                Action::ReturnZomeFunctionResult(ref result) => {
                    new_nucleus_state
                        .ribosome_calls
                        .insert(result.call.clone(), Some(result.result.clone()));
                }

                // Validate an Entry by calling its validation function
                Action::ValidateEntry(ref es) => {
                    let mut _has_entry_type = false;

                    // must have entry_type
                    if let Some(ref dna) = new_nucleus_state.dna {
                        if let Some(ref _wasm) =
                            dna.get_validation_bytecode_for_entry_type(&es.zome_name, &es.type_name)
                        {
                            // TODO #61 validate()
                            // Do same thing as Action::ExecuteZomeFunction
                            _has_entry_type = true;
                        }
                    }
                }
            }
            Arc::new(new_nucleus_state)
        }
        _ => old_state,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{nucleus::Action::*, state::Action::*}, *,
    };
    use std::sync::mpsc::channel;

    #[test]
    fn can_instantiate_nucleus_state() {
        let nucleus_state = NucleusState::new();
        assert_eq!(nucleus_state.dna, None);
        assert_eq!(nucleus_state.has_initialized(), false);
        assert_eq!(nucleus_state.status(), NucleusStatus::New);
    }

    #[test]
    fn can_reduce_initialize_action() {
        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, receiver) = channel::<state::ActionWrapper>();
        let (tx_observer, _observer) = channel::<Observer>();

        // Reduce Init action and block until receiving ReturnInit Action
        let reduced_nucleus = reduce(
            nucleus.clone(),
            &action,
            &sender.clone(),
            &tx_observer.clone(),
        );
        receiver.recv().unwrap_or_else(|_| panic!("channel failed"));

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);
    }

    #[test]
    fn can_reduce_return_init_result_action() {
        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, receiver) = channel::<state::ActionWrapper>();
        let (tx_observer, _observer) = channel::<Observer>();

        // Reduce Init action and block until receiving ReturnInit Action
        let reduced_nucleus = reduce(
            nucleus.clone(),
            &action,
            &sender.clone(),
            &tx_observer.clone(),
        );
        receiver.recv().unwrap_or_else(|_| panic!("receiver fail"));

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(false) Action
        let return_action = Nucleus(ReturnInitializationResult(false));
        let reduced_nucleus = reduce(
            reduced_nucleus.clone(),
            &return_action,
            &sender.clone(),
            &tx_observer.clone(),
        );

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::New);

        // Reduce Init action and block until receiving ReturnInit Action
        let reduced_nucleus = reduce(
            reduced_nucleus.clone(),
            &action,
            &sender.clone(),
            &tx_observer.clone(),
        );
        receiver.recv().unwrap_or_else(|_| panic!("receiver fail"));

        // Send ReturnInit(true) Action
        let return_action = Nucleus(ReturnInitializationResult(true));
        let reduced_nucleus = reduce(
            reduced_nucleus.clone(),
            &return_action,
            &sender.clone(),
            &tx_observer.clone(),
        );

        assert_eq!(reduced_nucleus.has_initialized(), true);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initialized);
    }

    #[test]
    fn can_reduce_execfn_action() {
        let call = FunctionCall::new(
            "myZome".to_string(),
            "public".to_string(),
            "bogusfn".to_string(),
            "".to_string(),
        );

        let action = Nucleus(ExecuteZomeFunction(call));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = channel::<state::ActionWrapper>();
        let (tx_observer, _observer) = channel::<Observer>();
        let reduced_nucleus = reduce(nucleus.clone(), &action, &sender, &tx_observer);
        assert_eq!(nucleus, reduced_nucleus);
    }
}
