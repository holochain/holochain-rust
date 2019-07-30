use crate::{
    action::{Action, ActionWrapper, GetEntryKey,GetLinksKey,Key,GetPayload,RespondGetPayload},
    context::Context,
    instance::dispatch_action,
    network::query::GetLinksNetworkQuery
};
use futures::{future::Future, task::Poll};

use holochain_persistence_api::cas::content::Address;

use holochain_core_types::{entry::EntryWithMetaAndHeader, error::HcResult, time::Timeout,crud_status::CrudStatus};

use std::{pin::Pin, sync::Arc, thread};

use snowflake::ProcessUniqueId;

use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, LinksStatusRequestKind};

/// FetchEntry Action Creator
/// This is the network version of get_entry that makes the network module start
/// a look-up process.
///
/// Returns a future that resolves to an ActionResponse.]

#[derive(Clone, PartialEq, Debug, Serialize)]
pub enum GetMethod
{
    Entry(Address),
    Link(GetLinksArgs,GetLinksNetworkQuery)
}

pub async fn get_entry(
    context: Arc<Context>,
    method : GetMethod,
    timeout: Timeout
) -> HcResult<RespondGetPayload> {

    let (key,payload) = match method 
    {
        GetMethod::Entry(address) =>{
            let key = GetEntryKey {
            address: address,
            id: snowflake::ProcessUniqueId::new().to_string()};
            (Key::Entry(key),GetPayload::Entry)
        },
        GetMethod::Link(link_args,query) =>
        {
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
            (Key::Links(key.clone()),GetPayload::Links((crud_status,query)))
        }
    };

    let entry = Action::Get((key.clone(),payload.clone()));
    let action_wrapper = ActionWrapper::new(entry);
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let key_inner = key.clone();
    let context_inner = context.clone();
    thread::Builder::new()
        .name(format!("get_entry_timeout/{:?}", key))
        .spawn(move || {
            thread::sleep(timeout.into());
            let timeout_action = Action::GetTimeout(key_inner);
            let action_wrapper = ActionWrapper::new(timeout_action);
            dispatch_action(context_inner.action_channel(), action_wrapper.clone());
        })
        .expect("Could not spawn thread for get_entry timeout");

    await!(GetFuture {
        context: context.clone(),
        key : key.clone(),
    })
}

/// GetEntryFuture resolves to a HcResult<Entry>.
/// Tracks the state of the network module
pub struct GetFuture {
    context: Arc<Context>,
    key: Key,
}

impl Future for GetFuture {
    type Output = HcResult<RespondGetPayload>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("GetEntryFuture") {
            return Poll::Ready(Err(err));
        }
        if let Err(error) = self
            .context
            .state()
            .expect("Could not get state  in future")
            .network()
            .initialized()
        {
            return Poll::Ready(Err(error));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        match self
            .context
            .state()
            .expect("Could not get state in future")
            .network()
            .get_results
            .get(&self.key)
        {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
