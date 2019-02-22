use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::entry_with_header::EntryWithHeader,
    workflows::author_entry::author_entry
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    error::HolochainError,
    crud_status::CrudStatus,
    entry::Entry

};
use std::{pin::Pin, sync::Arc};

pub async fn hold_entry<'a>(
    entry_wh: EntryWithHeader,
    context: Arc<Context>,
) -> Result<Address, HolochainError> {
    let action_wrapper = ActionWrapper::new(Action::Hold(entry_wh.clone()));
    let new_context = context.clone();
    let entry = entry_wh.entry.clone();
    await!(author_entry(&entry,None,&new_context))?;
    await!(HoldEntryFuture {
        context: context,
        address: entry_wh.entry.address(),
    })
}

pub struct HoldEntryFuture {
    context: Arc<Context>,
    address: Address,
}

impl Future for HoldEntryFuture {
    type Output = Result<Address, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
            if state
                .dht()
                .content_storage()
                .read()
                .unwrap()
                .contains(&self.address)
                .unwrap()
            {
                Poll::Ready(Ok(self.address.clone()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
