use crate::{
    action::{Action, ActionWrapper, GetEntryKey},
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::Address, chain_header::ChainHeader, entry::EntryWithMetaAndHeader, error::HcResult,
    time::Timeout
};
use std::{pin::Pin, sync::Arc, thread};

/// FetchEntry Action Creator
/// This is the network version of get_entry that makes the network module start
/// a look-up process.
///
/// Returns a future that resolves to an ActionResponse.
pub async fn get_entry(
    context: Arc<Context>,
    address: Address,
    timeout: Timeout,
) -> HcResult<Option<EntryWithMetaAndHeader>> {
    let key = GetEntryKey {
        address: address,
        id: snowflake::ProcessUniqueId::new().to_string(),
    };

    let action_wrapper = ActionWrapper::new(Action::FetchEntry(key.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let key_inner = key.clone();
    let context_inner = context.clone();
    let _ = thread::spawn(move || {
        thread::sleep(timeout.into());
        let action_wrapper = ActionWrapper::new(Action::GetEntryTimeout(key_inner));
        dispatch_action(context_inner.action_channel(), action_wrapper.clone());
    });

    await!(GetEntryFuture {
        context: context.clone(),
        key
    })
}

/// GetEntryFuture resolves to a HcResult<Entry>.
/// Tracks the state of the network module
pub struct GetEntryFuture {
    context: Arc<Context>,
    key: GetEntryKey,
}

impl Future for GetEntryFuture {
    type Output = HcResult<Option<EntryWithMetaAndHeader>>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        if let Err(error) = self.context.state().expect("Could not get state  in future").network().initialized() {
            return Poll::Ready(Err(error));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        match self.context.state().expect("Could not get state in future").network().get_entry_with_meta_results.get(&self.key) {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
