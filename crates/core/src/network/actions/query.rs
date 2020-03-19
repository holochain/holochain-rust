use crate::{
    action::{Action, ActionWrapper, GetEntryKey, GetLinksKey, QueryKey, QueryPayload},
    context::Context,
    instance::dispatch_action,
    network::query::{GetLinksNetworkQuery, NetworkQueryResult},
};
use futures::{future::Future, task::Poll};

use holochain_persistence_api::cas::content::Address;

use holochain_core_types::{crud_status::CrudStatus, error::HcResult, time::Timeout};

use std::{pin::Pin, sync::Arc};

use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, LinksStatusRequestKind};
use std::time::SystemTime;

/// FetchEntry Action Creator
/// This is the network version of get_entry that makes the network module start
/// a look-up process.
///
/// Returns a future that resolves to an ActionResponse.]

#[derive(Clone, PartialEq, Debug, Serialize)]
pub enum QueryMethod {
    Entry(Address),
    Link(GetLinksArgs, GetLinksNetworkQuery),
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn query(
    context: Arc<Context>,
    method: QueryMethod,
    timeout: Timeout,
) -> HcResult<NetworkQueryResult> {
    let (key, payload) = match method {
        QueryMethod::Entry(address) => {
            let key = GetEntryKey {
                address,
                id: nanoid::simple(),
            };
            (QueryKey::Entry(key), QueryPayload::Entry)
        }
        QueryMethod::Link(link_args, query) => {
            let key = GetLinksKey {
                base_address: link_args.entry_address.clone(),
                link_type: link_args.link_type.clone(),
                tag: link_args.tag.clone(),
                id: nanoid::simple(),
            };
            let crud_status = match link_args.options.status_request {
                LinksStatusRequestKind::All => None,
                LinksStatusRequestKind::Deleted => Some(CrudStatus::Deleted),
                LinksStatusRequestKind::Live => Some(CrudStatus::Live),
            };
            (
                QueryKey::Links(key),
                QueryPayload::Links((crud_status, query)),
            )
        }
    };

    let entry = Action::Query((
        key.clone(),
        payload.clone(),
        Some((SystemTime::now(), timeout.into())),
    ));
    let action_wrapper = ActionWrapper::new(entry);
    dispatch_action(context.action_channel(), action_wrapper.clone());
    QueryFuture {
        context: context.clone(),
        key: key.clone(),
    }
    .await
}

/// GetEntryFuture resolves to a HcResult<Entry>.
/// Tracks the state of the network module
pub struct QueryFuture {
    context: Arc<Context>,
    key: QueryKey,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for QueryFuture {
    type Output = HcResult<NetworkQueryResult>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("GetEntryFuture") {
            return Poll::Ready(Err(err));
        }

        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();

        if self.context.action_channel().is_full() {
            return Poll::Pending;
        }
        if let Some(state) = self.context.try_state() {
            if let Err(error) = state.network().initialized() {
                return Poll::Ready(Err(error));
            }
            match state.network().get_query_results.get(&self.key) {
                Some(Some(result)) => {
                    if self.context.action_channel().is_full() {
                        return Poll::Pending;
                    }
                    dispatch_action(
                        self.context.action_channel(),
                        ActionWrapper::new(Action::ClearQueryResult(self.key.clone())),
                    );
                    Poll::Ready(result.clone())
                }
                _ => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
