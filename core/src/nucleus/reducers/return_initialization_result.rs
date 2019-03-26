use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::state::{NucleusState, NucleusStatus},
};
use std::sync::Arc;

/// Reduce ReturnInitializationResult Action
/// On initialization success, set Initialized status
/// otherwise set the failed message
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
pub fn reduce_return_initialization_result(
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
            Ok(init) => state.status = NucleusStatus::Initialized(init.clone()),
            Err(err) => state.status = NucleusStatus::InitializationFailed(err.clone()),
        };
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        action::ActionWrapper,
        instance::{tests::test_context_with_channels, Observer},
        nucleus::{
            actions::initialize::Initialization,
            reduce,
            state::{NucleusState, NucleusStatus},
        },
    };
    use holochain_core_types::dna::Dna;
    use std::sync::{mpsc::sync_channel, Arc};

    #[test]
    /// test that we can initialize and send/receive result values from a nucleus
    fn can_reduce_return_init_result_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(Action::InitializeChain(dna));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = sync_channel::<ActionWrapper>(10);
        let (tx_observer, _observer) = sync_channel::<Observer>(10);
        let context = test_context_with_channels("jimmy", &sender, &tx_observer, None).clone();

        // Reduce Init action
        let initializing_nucleus = reduce(context.clone(), nucleus.clone(), &action_wrapper);

        assert_eq!(initializing_nucleus.has_initialized(), false);
        assert_eq!(initializing_nucleus.has_initialization_failed(), false);
        assert_eq!(initializing_nucleus.initialization().is_some(), false);
        assert_eq!(initializing_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(failed) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(Action::ReturnInitializationResult(Err(
            "init failed".to_string(),
        )));
        let reduced_nucleus = reduce(
            context.clone(),
            initializing_nucleus.clone(),
            &return_action_wrapper,
        );

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.has_initialization_failed(), true);
        assert_eq!(reduced_nucleus.initialization().is_some(), false);
        assert_eq!(
            reduced_nucleus.status(),
            NucleusStatus::InitializationFailed("init failed".to_string())
        );

        // Reduce Init action
        let reduced_nucleus = reduce(context.clone(), reduced_nucleus.clone(), &action_wrapper);

        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(None) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(Action::ReturnInitializationResult(Ok(
            Initialization::new(),
        )));
        let reduced_nucleus = reduce(
            context.clone(),
            initializing_nucleus.clone(),
            &return_action_wrapper,
        );

        assert_eq!(reduced_nucleus.has_initialized(), true);
        assert_eq!(reduced_nucleus.has_initialization_failed(), false);
        assert_eq!(reduced_nucleus.initialization().is_some(), true);
        assert_eq!(
            reduced_nucleus.status(),
            NucleusStatus::Initialized(Initialization::new())
        );
    }
}
