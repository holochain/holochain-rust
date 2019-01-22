/// Nucleus is the module that handles DNA, including the Ribosome.
///
pub mod actions;
pub mod ribosome;
pub mod state;

use crate::{
    action::{Action, ActionWrapper, NucleusReduceFn},
    context::Context,
    nucleus::{
        ribosome::{api::call::reduce_call,
                   fn_call::{reduce_execute_zome_function,reduce_return_zome_function_result,}
                   },
        state::{NucleusState, NucleusStatus},
    },
};
use std::sync::Arc;

/// Reduce ReturnInitializationResult Action
/// On initialization success, set Initialized status
/// otherwise set the failed message
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_return_initialization_result(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
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

/// Reduce InitApplication Action
/// Switch status to failed if an initialization is tried for an
/// already initialized, or initializing instance.
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
fn reduce_init_application(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    match state.status() {
        NucleusStatus::Initializing => {
            state.status =
                NucleusStatus::InitializationFailed("Nucleus already initializing".to_string())
        }
        NucleusStatus::Initialized => {
            state.status =
                NucleusStatus::InitializationFailed("Nucleus already initialized".to_string())
        }
        NucleusStatus::New | NucleusStatus::InitializationFailed(_) => {
            let ia_action = action_wrapper.action();
            let dna = unwrap_to!(ia_action => Action::InitApplication);
            // Update status
            state.status = NucleusStatus::Initializing;
            // Set DNA
            state.dna = Some(dna.clone());
        }
    }
}

fn reduce_return_validation_result(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let ((id, hash), validation_result) = unwrap_to!(action => Action::ReturnValidationResult);
    state
        .validation_results
        .insert((id.clone(), hash.clone()), validation_result.clone());
}

fn reduce_return_validation_package(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (id, maybe_validation_package) = unwrap_to!(action => Action::ReturnValidationPackage);
    state
        .validation_packages
        .insert(id.clone(), maybe_validation_package.clone());
}

/// Maps incoming action to the correct reducer
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NucleusReduceFn> {
    match action_wrapper.action() {
        Action::ReturnInitializationResult(_) => Some(reduce_return_initialization_result),
        Action::InitApplication(_) => Some(reduce_init_application),
        Action::ExecuteZomeFunction(_) => Some(reduce_execute_zome_function),
        Action::ReturnZomeFunctionResult(_) => Some(reduce_return_zome_function_result),
        Action::Call(_) => Some(reduce_call),
        Action::ReturnValidationResult(_) => Some(reduce_return_validation_result),
        Action::ReturnValidationPackage(_) => Some(reduce_return_validation_package),
        _ => None,
    }
}

/// Reduce state of Nucleus according to action.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<NucleusState>,
    action_wrapper: &ActionWrapper,
) -> Arc<NucleusState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: NucleusState = (*old_state).clone();
            f(context, &mut new_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    use super::*;
    use crate::{
        action::ActionWrapper,
        instance::{
            tests::{test_context_with_channels},
            Observer,
        },
    };
    use holochain_core_types::{
        dna::{
            Dna,
        },
    };
    use std::sync::Arc;
    use std::sync::mpsc::sync_channel;

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
    /// smoke test the init of a nucleus reduction
    fn can_reduce_initialize_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(Action::InitApplication(dna.clone()));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = sync_channel::<ActionWrapper>(10);
        let (tx_observer, _observer) = sync_channel::<Observer>(10);
        let context = test_context_with_channels("jimmy", &sender, &tx_observer, None);

        // Reduce Init action
        let reduced_nucleus = reduce(context.clone(), nucleus.clone(), &action_wrapper);

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.has_initialization_failed(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);
        assert!(reduced_nucleus.dna().is_some());
        assert_eq!(reduced_nucleus.dna().unwrap(), dna);
    }

    #[test]
    /// test that we can initialize and send/receive result values from a nucleus
    fn can_reduce_return_init_result_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(Action::InitApplication(dna));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = sync_channel::<ActionWrapper>(10);
        let (tx_observer, _observer) = sync_channel::<Observer>(10);
        let context = test_context_with_channels("jimmy", &sender, &tx_observer, None).clone();

        // Reduce Init action
        let initializing_nucleus = reduce(context.clone(), nucleus.clone(), &action_wrapper);

        assert_eq!(initializing_nucleus.has_initialized(), false);
        assert_eq!(initializing_nucleus.has_initialization_failed(), false);
        assert_eq!(initializing_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(false) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(Action::ReturnInitializationResult(Some(
            "init failed".to_string(),
        )));
        let reduced_nucleus = reduce(
            context.clone(),
            initializing_nucleus.clone(),
            &return_action_wrapper,
        );

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.has_initialization_failed(), true);
        assert_eq!(
            reduced_nucleus.status(),
            NucleusStatus::InitializationFailed("init failed".to_string())
        );

        // Reduce Init action
        let reduced_nucleus = reduce(context.clone(), reduced_nucleus.clone(), &action_wrapper);

        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(None) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(Action::ReturnInitializationResult(None));
        let reduced_nucleus = reduce(
            context.clone(),
            initializing_nucleus.clone(),
            &return_action_wrapper,
        );

        assert_eq!(reduced_nucleus.has_initialized(), true);
        assert_eq!(reduced_nucleus.has_initialization_failed(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initialized);
    }
}
