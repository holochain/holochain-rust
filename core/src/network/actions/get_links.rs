use crate::{
    action::{Action, ActionWrapper, GetLinksKey},
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{crud_status::CrudStatus, error::HcResult, time::Timeout};
use holochain_persistence_api::cas::content::Address;
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc, thread};

/// GetLinks Action Creator
/// This is the network version of get_links that makes the network module start
/// a look-up process.
pub async fn get_links(
    context: Arc<Context>,
    address: Address,
    link_type: String,
    tag: String,
    timeout: Timeout,
) -> HcResult<Vec<(Address, CrudStatus)>> {
    let key = GetLinksKey {
        base_address: address.clone(),
        link_type: link_type.clone(),
        tag: tag.clone(),
        id: ProcessUniqueId::new().to_string(),
    };
    let action_wrapper = ActionWrapper::new(Action::GetLinks(key.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let key_inner = key.clone();
    let context_inner = context.clone();
    let _ = thread::spawn(move || {
        thread::sleep(timeout.into());
        let action_wrapper = ActionWrapper::new(Action::GetLinksTimeout(key_inner));
        dispatch_action(context_inner.action_channel(), action_wrapper.clone());
    });

    await!(GetLinksFuture {
        context: context.clone(),
        key
    })
}

/// GetLinksFuture resolves to a HcResult<Vec<Address>>.
/// Tracks the state of the network module
pub struct GetLinksFuture {
    context: Arc<Context>,
    key: GetLinksKey,
}

impl Future for GetLinksFuture {
    type Output = HcResult<Vec<(Address, CrudStatus)>>;

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
        match state.get_links_results.get(&self.key) {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
