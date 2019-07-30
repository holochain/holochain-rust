use crate::{
    action::{Action, ActionWrapper, GetLinksKey,Key,GetPayload},
    context::Context,
    instance::dispatch_action,
    network::query::{GetLinksNetworkQuery, GetLinksNetworkResult},
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{crud_status::CrudStatus, error::HcResult};
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc, thread};

use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, LinksStatusRequestKind};

/// GetLinks Action Creator
/// This is the network version of get_links that makes the network module start
/// a look-up process.
pub async fn get_links(
    context: Arc<Context>,
    link_args: &GetLinksArgs,
    query: GetLinksNetworkQuery,
) -> HcResult<GetLinksNetworkResult> {
    let key = GetLinksKey {
        base_address: link_args.entry_address.clone(),
        link_type: link_args.link_type.clone(),
        tag: link_args.tag.clone(),
        id: ProcessUniqueId::new().to_string(),
    };
    let crud_status = match link_args.options.status_request {
        LinksStatusRequestKind::All => None,
        LinksStatusRequestKind::Deleted => Some(CrudStatus::Deleted),
        LinksStatusRequestKind::Live => Some(CrudStatus::Live),
    };
    let get_action = Action::Get((Key::Links(key.clone()),GetPayload::Links((crud_status,query))));
    let action_wrapper = ActionWrapper::new(get_action);
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let key_inner = key.clone();
    let context_inner = context.clone();
    let timeout = link_args.options.timeout.clone();
    thread::Builder::new()
        .name(format!("get_links/{:?}", key))
        .spawn(move || {
            thread::sleep(timeout.into());
            let get_links_timeout = Action::GetTimeout(Key::Links(key_inner));
            let action_wrapper = ActionWrapper::new(get_links_timeout);
            dispatch_action(context_inner.action_channel(), action_wrapper.clone());
        })
        .expect("Could not spawn thread for get_links timeout");

    await!(GetLinksFuture {
        context: context.clone(),
        key,
    })
}

/// GetLinksFuture resolves to a HcResult<Vec<Address>>.
/// Tracks the state of the network module
pub struct GetLinksFuture {
    context: Arc<Context>,
    key: GetLinksKey,
}

impl Future for GetLinksFuture {
    type Output = HcResult<GetLinksNetworkResult>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("GetLinksFuture") {
            return Poll::Ready(Err(err));
        }
        let state = self.context.state().unwrap().network();
        if let Err(error) = state.initialized() {
            return Poll::Ready(Err(error));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        match state.get_links_results.get(&self.key) {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
