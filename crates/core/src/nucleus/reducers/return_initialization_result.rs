use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::{NucleusState, NucleusStatus},
    state::State,NEW_RELIC_LICENSE_KEY
};

/// Reduce ReturnInitializationResult Action
/// On initialization success, set Initialized status
/// otherwise set the failed message
#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_return_initialization_result(
    state: &mut NucleusState,
    _root_state: &State,
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
        state::test_store,
    };
    use crossbeam_channel::unbounded;
    use holochain_core_types::dna::Dna;
    use std::sync::Arc;

    #[test]
    /// test that we can initialize and send/receive result values from a nucleus
    fn can_reduce_return_init_result_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(Action::InitializeChain(dna));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = unbounded::<ActionWrapper>();
        let (tx_observer, _observer) = unbounded::<Observer>();
        let context = test_context_with_channels("jimmy", &sender, &tx_observer, None).clone();
        let root_state = test_store(context);

        // Reduce Init action
        let initializing_nucleus = reduce(nucleus.clone(), &root_state, &action_wrapper);

        assert_eq!(initializing_nucleus.has_initialized(), false);
        assert_eq!(initializing_nucleus.has_initialization_failed(), false);
        assert_eq!(initializing_nucleus.initialization().is_some(), false);
        assert_eq!(initializing_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(failed) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(Action::ReturnInitializationResult(Err(
            "init failed".to_string(),
        )));
        let reduced_nucleus = reduce(
            initializing_nucleus.clone(),
            &root_state,
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
        let reduced_nucleus = reduce(reduced_nucleus.clone(), &root_state, &action_wrapper);

        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);

        // Send ReturnInit(None) ActionWrapper
        let return_action_wrapper = ActionWrapper::new(Action::ReturnInitializationResult(Ok(
            Initialization::new(),
        )));
        let reduced_nucleus = reduce(
            initializing_nucleus.clone(),
            &root_state,
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
