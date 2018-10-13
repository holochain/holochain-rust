extern crate serde_json;
use context::Context;
use futures::{future, Future};
use holochain_core_types::{
    cas::{content::Address, storage::ContentAddressableStorage},
    entry::Entry,
    error::HolochainError,
};
use std::sync::Arc;

fn get_entry_from_dht_cas(
    context: &Arc<Context>,
    address: Address,
) -> Result<Option<Entry>, HolochainError> {
    let dht = context.state().unwrap().dht().content_storage();
    dht.fetch(&address)
}

/// GetEntry Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn get_entry(
    context: &Arc<Context>,
    address: Address,
) -> Box<dyn Future<Item = Option<Entry>, Error = HolochainError>> {
    match get_entry_from_dht_cas(context, address) {
        Err(err) => Box::new(future::err(err)),
        Ok(result) => Box::new(future::ok(result)),
    }
}

#[cfg(test)]
pub mod tests {
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::{content::AddressableContent, storage::ContentAddressableStorage},
        entry::test_entry,
    };
    use instance::tests::test_context_with_state;

    #[test]
    fn get_entry_from_dht_cas() {
        let entry = test_entry();
        let context = test_context_with_state();
        let result = super::get_entry_from_dht_cas(&context, entry.address());
        assert_eq!(Ok(None), result);
        context
            .state()
            .unwrap()
            .dht()
            .content_storage()
            .add(&entry)
            .unwrap();
        let result = super::get_entry_from_dht_cas(&context, entry.address());
        assert_eq!(Ok(Some(entry.clone())), result);
    }

    #[test]
    fn get_entry_from_dht_cas_futures() {
        let entry = test_entry();
        let context = test_context_with_state();
        let future = super::get_entry(&context, entry.address());
        assert_eq!(Ok(None), block_on(future));
        context
            .state()
            .unwrap()
            .dht()
            .content_storage()
            .add(&entry)
            .unwrap();
        let future = super::get_entry(&context, entry.address());
        assert_eq!(Ok(Some(entry.clone())), block_on(future));
    }

}
