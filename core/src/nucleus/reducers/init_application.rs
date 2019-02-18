use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::state::{NucleusState, NucleusStatus},
};
use std::sync::Arc;

/// Reduce InitApplication Action
/// Switch status to failed if an initialization is tried for an
/// already initialized, or initializing instance.
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
pub fn reduce_init_application(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    match state.status() {
        NucleusStatus::Initializing => {
            state.status =
                NucleusStatus::InitializationFailed("Nucleus already initializing".to_string())
        }
        NucleusStatus::Initialized(_) => {
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

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    use super::*;
    use crate::{
        action::ActionWrapper,
        instance::{tests::test_context_with_channels, Observer},
        nucleus::{
            reduce,
            state::{NucleusState, NucleusStatus},
        },
    };
    use holochain_core_types::dna::Dna;
    use std::sync::{mpsc::sync_channel, Arc};

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
}
