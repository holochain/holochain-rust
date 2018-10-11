extern crate serde_json;
use action::{Action, ActionWrapper};
use context::Context;
use futures::{future, Async, Future};
use holochain_core_types::{
    error::HolochainError, cas::content::AddressableContent, entry::Entry, entry_type::EntryType, cas::content::Address};
use holochain_wasm_utils::validation::ValidationData;
use nucleus::ribosome::callback::{self, CallbackResult};
use snowflake;
use std::{sync::Arc, thread};
use holochain_core_types::cas::storage::ContentAddressableStorage;

fn get_entry_from_dht_cas(address: Address, context: &Arc<Context>) -> Result<Option<Entry>,HolochainError> {
    let dht = context.state().unwrap().dht().content_storage();
    dht.fetch(&address)
}

/// GetEntry Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn get_entry(
    address: Address,
    context: &Arc<Context>,
) -> Box<dyn Future<Item = Option<Entry>, Error = String>> {
    match get_entry_from_dht_cas(address, context) {
        Err(err) => Box::new(future::err(err.to_string())),
        Ok(result) => Box::new(future::ok(result))
    }
}

#[cfg(test)]
pub mod tests {
    use holochain_core_types::{
        entry::{test_entry},
        cas::content::AddressableContent,
        cas::storage::ContentAddressableStorage,
    };
    use instance::tests::test_context_with_state;
    use  futures::executor::block_on;

    #[test]
    fn get_entry_from_dht_cas() {
        let entry = test_entry();
        let context = test_context_with_state();
        let result = super::get_entry_from_dht_cas(entry.address(), &context);
        assert_eq!(Ok(None), result);
        context.state().unwrap().dht().content_storage().add(&entry).unwrap();
        let result = super::get_entry_from_dht_cas(entry.address(), &context);
        assert_eq!(Ok(Some(entry.clone())),result);
    }

    #[test]
    fn get_entry_from_dht_cas_futures() {
        let entry = test_entry();
        let context = test_context_with_state();
        let future = super::get_entry(entry.address(), &context);
        assert_eq!(Ok(None),block_on(future));
        context.state().unwrap().dht().content_storage().add(&entry).unwrap();
        let future = super::get_entry(entry.address(), &context);
        assert_eq!(Ok(Some(entry.clone())),block_on(future));
    }

}
