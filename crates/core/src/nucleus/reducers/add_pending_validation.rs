use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::{NucleusState, PendingValidationKey},
    state::State,
};
use holochain_persistence_api::cas::content::AddressableContent;

/// Reduce AddPendingValidation Action.
/// Inserts boxed EntryWithHeader and dependencies into state, referenced with
/// the entry's address.
#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
pub fn reduce_add_pending_validation(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let pending = unwrap_to!(action => Action::AddPendingValidation);
    let address = pending.chain_pair.entry().address();
    let workflow = pending.workflow.clone();
    state.pending_validations.insert(
        PendingValidationKey::new(address, workflow),
        pending.clone(),
    );
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        instance::tests::test_context,
        network::chain_pair::ChainPair,
        nucleus::state::{tests::test_nucleus_state, PendingValidationKey},
        scheduled_jobs::pending_validations::{PendingValidationStruct, ValidatingWorkflow},
        state::test_store,
    };
    use holochain_core_types::{chain_header::test_chain_header, entry::Entry};
    use holochain_json_api::json::RawString;
    use std::sync::Arc;

    #[test]
    fn test_reduce_add_pending_validation() {
        let context = test_context("jimmy", None);
        let mut state = test_nucleus_state();
        let root_state = test_store(context);

        let entry = Entry::App("package_entry".into(), RawString::from("test value").into());
        let chain_pair = ChainPair::new(test_chain_header(), entry.clone());

        let action_wrapper = ActionWrapper::new(Action::AddPendingValidation(Arc::new(
            PendingValidationStruct {
                chain_pair,
                dependencies: Vec::new(),
                workflow: ValidatingWorkflow::HoldEntry,
            },
        )));

        reduce_add_pending_validation(&mut state, &root_state, &action_wrapper);

        assert!(state
            .pending_validations
            .contains_key(&PendingValidationKey::new(
                entry.address(),
                ValidatingWorkflow::HoldEntry
            )));
    }
}
