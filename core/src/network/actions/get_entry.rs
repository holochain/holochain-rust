extern crate futures;
use crate::{
    action::{Action, ActionWrapper, GetEntryKey},
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{cas::content::Address, entry::EntryWithMeta, error::HcResult};
use std::{pin::Pin, sync::Arc, thread::sleep, time::Duration};

/// FetchEntry Action Creator
/// This is the network version of get_entry that makes the network module start
/// a look-up process.
///
/// Returns a future that resolves to an ActionResponse.
pub async fn get_entry<'a>(
    context: &'a Arc<Context>,
    address: &'a Address,
) -> HcResult<Option<EntryWithMeta>> {
    let key = GetEntryKey {
        address: address.clone(),
        id: snowflake::ProcessUniqueId::new().to_string(),
    };

    let action_wrapper = ActionWrapper::new(Action::FetchEntry(key.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let _ = async {
        sleep(Duration::from_secs(60));
        let action_wrapper = ActionWrapper::new(Action::GetEntryTimeout(key.clone()));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    };

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
    type Output = HcResult<Option<EntryWithMeta>>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        let state = self.context.state().unwrap().network();
        if let Err(error) = state.initialized() {
            return Poll::Ready(Err(error));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        match state.get_entry_with_meta_results.get(&self.key) {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
