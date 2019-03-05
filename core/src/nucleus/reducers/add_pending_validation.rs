use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::state::{NucleusState, PendingValidationKey},
};
use holochain_core_types::cas::content::AddressableContent;
use std::sync::Arc;

/// Reduce AddPendingValidation Action.
/// Inserts boxed EntryWithHeader and dependencies into state, referenced with
/// the entry's address.
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
pub fn reduce_add_pending_validation(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let pending = unwrap_to!(action => Action::AddPendingValidation);
    let address = pending.entry_with_header.entry.address();
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
        network::entry_with_header::EntryWithHeader,
        nucleus::state::tests::test_nucleus_state,
        scheduled_jobs::pending_validations::{PendingValidationStruct, ValidatingWorkflow},
    };
    use holochain_core_types::{chain_header::test_chain_header, entry::Entry, json::RawString};

    #[test]
    fn test_reduce_add_pending_validation() {
        let context = test_context("jimmy", None);
        let mut state = test_nucleus_state();

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

        reduce_add_pending_validation(context, &mut state, &action_wrapper);

        assert!(state
            .pending_validations
            .contains_key(&(entry.address(), ValidatingWorkflow::HoldEntry)));
    }
}
