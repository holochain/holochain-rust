use crate::action::ActionWrapper;
use crate::network::state::NetworkState;
use crate::context::Context;
use std::sync::Arc;
use holochain_core_types::error::HolochainError;
use holochain_core_types::entry::Entry;
use crate::action::Action;

fn entry_to_cas(entry: &Entry, context: &Arc<Context>,) -> Result<(), HolochainError>{
    let mut cas = context.file_storage.write()?;
    Ok(cas.add(entry)?)
}

pub fn reduce_receive(
    context: Arc<Context>,
    _state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {

    let action = action_wrapper.action();
    let address = unwrap_to!(action => Action::Receive);

    let result = entry_to_cas(address, &context);
    if result.is_err() {
        return;
    };

}

#[cfg(test)]
mod tests {

    use crate::action::ActionWrapper;
    use crate::action::Action;
    use holochain_core_types::entry::test_entry;
    use crate::instance::tests::test_context;
    use crate::state::test_store;
    use holochain_core_types::cas::content::AddressableContent;
    use holochain_core_types::entry::SerializedEntry;
    use std::convert::TryFrom;

    #[test]
    pub fn reduce_receive_test() {
        let context = test_context("bill");
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::Receive(entry.clone()));

        store.reduce(
            context.clone(),
            action_wrapper,
        );

        let cas = context.file_storage.read().unwrap();

        let maybe_json = cas.fetch(&entry.address()).unwrap();
        let result_entry = match maybe_json {
            Some(content) => SerializedEntry::try_from(content).unwrap().deserialize(),
            None => panic!("Could not find received entry in CAS"),
        };

        assert_eq!(
            &entry,
            &result_entry,
        );
    }

}
