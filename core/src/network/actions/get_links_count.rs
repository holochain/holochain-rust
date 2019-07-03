use crate::{
    action::{Action, ActionWrapper, GetLinksKey,GetLinksKeyByTag},
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{crud_status::CrudStatus, error::HcResult, time::Timeout};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::get_links::LinksStatusRequestKind;
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc, thread};

/// GetLinks Action Creator
/// This is the network version of get_links that makes the network module start
/// a look-up process.
pub async fn get_links_count(
    context: Arc<Context>,
    address: Address,
    link_type: String,
    tag: String,
    timeout: Timeout,
    link_status_request: LinksStatusRequestKind,
) -> HcResult<usize> {
    let key = GetLinksKey {
        base_address: address.clone(),
        link_type: link_type.clone(),
        tag: tag.clone(),
        id: ProcessUniqueId::new().to_string(),
    };

    let crud_status = match link_status_request {
        LinksStatusRequestKind::All => None,
        LinksStatusRequestKind::Live => Some(CrudStatus::Live),
        LinksStatusRequestKind::Deleted => Some(CrudStatus::Deleted),
    };
    let action_wrapper = ActionWrapper::new(Action::GetLinksCount((key.clone(), crud_status)));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let key_inner = key.clone();
    let context_inner = context.clone();
    let _ = thread::spawn(move || {
        thread::sleep(timeout.into());
        let action_wrapper = ActionWrapper::new(Action::GetLinksTimeout(key_inner));
        dispatch_action(context_inner.action_channel(), action_wrapper.clone());
    });

    await!(GetLinksCountFuture {
        context: context.clone(),
        key
    })
}


pub async fn get_links_count_by_tag(
    context: Arc<Context>,
    tag: String,
    _timeout: Timeout,
    link_status_request: LinksStatusRequestKind,
) -> HcResult<usize> {
    let key = GetLinksKeyByTag {
        tag: tag.clone(),
        id: ProcessUniqueId::new().to_string(),
    };

    let crud_status = match link_status_request {
        LinksStatusRequestKind::All => None,
        LinksStatusRequestKind::Live => Some(CrudStatus::Live),
        LinksStatusRequestKind::Deleted => Some(CrudStatus::Deleted),
    };
    let action_wrapper = ActionWrapper::new(Action::GetLinksCountByTag((key.clone(), crud_status)));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    /*let key_inner = key.clone();
    let context_inner = context.clone();
    let _ = thread::spawn(move || {
        thread::sleep(timeout.into());
        let action_wrapper = ActionWrapper::new(Action::GetLinksTimeout(key_inner));
        dispatch_action(context_inner.action_channel(), action_wrapper.clone());
    });*/

    await!(GetLinksCountByTagFuture {
        context: context.clone(),
        key 
    })
}

/// GetLinksFuture resolves to a HcResult<Vec<Address>>.
/// Tracks the state of the network module
pub struct GetLinksCountFuture {
    context: Arc<Context>,
    key: GetLinksKey,
}

impl Future for GetLinksCountFuture {
    type Output = HcResult<usize>;

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
        match state.get_links_results_count.get(&self.key) {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}

/// GetLinksFuture resolves to a HcResult<Vec<Address>>.
/// Tracks the state of the network module
pub struct GetLinksCountByTagFuture {
    context: Arc<Context>,
    key: GetLinksKeyByTag,
}

impl Future for GetLinksCountByTagFuture {
    type Output = HcResult<usize>;

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
        match state.get_links_result_count_by_tag.get(&self.key) {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
