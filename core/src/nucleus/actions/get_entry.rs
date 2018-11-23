extern crate serde_json;
use crate::context::Context;
use futures::{future, Future};
use holochain_core_types::{
    cas::content::Address,
    entry::{Entry, SerializedEntry},
    error::HolochainError,
};
use std::{convert::TryInto, sync::Arc};

fn get_entry_from_dht_cas(
    context: &Arc<Context>,
    address: Address,
) -> Result<Option<Entry>, HolochainError> {
    let dht = context.state().unwrap().dht().content_storage();
    let storage = &dht.clone();
    let json = (*storage.read().unwrap()).fetch(&address)?;
    let entry: Option<Entry> = json
        .and_then(|js| js.try_into().ok())
        .map(|s: SerializedEntry| s.into());
    Ok(entry)
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
    use holochain_core_types::{cas::content::AddressableContent, entry::test_entry};
    use crate::instance::tests::test_context_with_state;

    #[test]
    fn get_entry_from_dht_cas() {
        let entry = test_entry();
        let context = test_context_with_state();
        let result = super::get_entry_from_dht_cas(&context, entry.address());
        assert_eq!(Ok(None), result);
        let storage = &context.state().unwrap().dht().content_storage().clone();
        (*storage.write().unwrap()).add(&entry).unwrap();
        let result = super::get_entry_from_dht_cas(&context, entry.address());
        assert_eq!(Ok(Some(entry.clone())), result);
    }

    #[test]
    fn get_entry_from_dht_cas_futures() {
        let entry = test_entry();
        let context = test_context_with_state();
        let future = super::get_entry(&context, entry.address());
        assert_eq!(Ok(None), block_on(future));
        let storage = &context.state().unwrap().dht().content_storage().clone();
        (*storage.write().unwrap()).add(&entry).unwrap();
        let future = super::get_entry(&context, entry.address());
        assert_eq!(Ok(Some(entry.clone())), block_on(future));
    }

}
