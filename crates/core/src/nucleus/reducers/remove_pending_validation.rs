use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::{NucleusState, PendingValidationKey},
    state::State,
};

/// Reduce RemovePendingValidation Action.
/// Removes boxed EntryWithHeader and dependencies from state, referenced with
/// the entry's address.
/// Corresponds to a prior AddPendingValidation Action.
#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
#[cfg(not(target_arch = "wasm32"))]
#[flame]
pub fn reduce_remove_pending_validation(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (address, workflow) = unwrap_to!(action => Action::RemovePendingValidation).clone();
    state
        .pending_validations
        .remove(&PendingValidationKey::new(address, workflow));
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        instance::tests::test_context,
        network::entry_with_header::EntryWithHeader,
        nucleus::{
            reducers::add_pending_validation::reduce_add_pending_validation,
            state::tests::test_nucleus_state,
        },
        scheduled_jobs::pending_validations::{PendingValidationStruct, ValidatingWorkflow},
        state::test_store,
    };
    use holochain_core_types::{chain_header::test_chain_header, entry::Entry};
    use holochain_json_api::json::RawString;
    use holochain_persistence_api::cas::content::AddressableContent;
    use std::sync::Arc;

    #[test]
    fn test_reduce_remove_pending_validation() {
        let context = test_context("jimmy", None);
        let mut nucleus_state = test_nucleus_state();
        let state = test_store(context);

        let entry = Entry::App("package_entry".into(), RawString::from("test value").into());
        let entry_with_header = EntryWithHeader {
            entry: entry.clone(),
            header: test_chain_header(),
        };

        let action_wrapper = ActionWrapper::new(Action::AddPendingValidation(Arc::new(
            PendingValidationStruct {
                entry_with_header,
                dependencies: Vec::new(),
                workflow: ValidatingWorkflow::HoldEntry,
            },
        )));

        reduce_add_pending_validation(&mut nucleus_state, &state, &action_wrapper);

        assert!(nucleus_state
            .pending_validations
            .contains_key(&PendingValidationKey::new(
                entry.address(),
                ValidatingWorkflow::HoldEntry
            )));

        let action_wrapper = ActionWrapper::new(Action::RemovePendingValidation((
            entry.address(),
            ValidatingWorkflow::HoldEntry,
        )));

        reduce_remove_pending_validation(&mut nucleus_state, &state, &action_wrapper);

        assert!(!nucleus_state
            .pending_validations
            .contains_key(&PendingValidationKey::new(
                entry.address(),
                ValidatingWorkflow::HoldEntry
            )));
    }
}
