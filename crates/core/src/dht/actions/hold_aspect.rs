use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{error::HolochainError, network::entry_aspect::EntryAspect};
use holochain_persistence_api::cas::content::AddressableContent;
use lib3h_protocol::data_types::EntryListData;
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc};

pub async fn hold_aspect(aspect: EntryAspect, context: Arc<Context>) -> Result<(), HolochainError> {
    let id = ProcessUniqueId::new();
    let action_wrapper = ActionWrapper::new(Action::HoldAspect((aspect.clone(), id)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    let r = HoldAspectFuture {
        context: context.clone(),
        //        aspect,
        id,
    }
    .await;
    if r.is_err() {
        panic!(r);
    } else {
        // send a gossip list with this aspect in it back to sim2h so it know we are holding it
        let c = context.clone();
        let closure = async move || {
            let state = context
                .state()
                .expect("No state present when trying to respond with gossip list");

            // TODO: send the whole holding map for now fix to the specific aspect later
            let address_map = state.dht().get_holding_map().clone();

            let action = Action::RespondGossipList(EntryListData {
                space_address: state.network().dna_address.clone().unwrap().into(),
                provider_agent_id: context.agent_id.address().into(), //get_list_data.provider_agent_id,
                request_id: "".to_string(),
                address_map: address_map.into(),
            });
            dispatch_action(context.action_channel(), ActionWrapper::new(action));
        };
        let future = closure();
        c.spawn_task(future);
    }
    r
}

pub struct HoldAspectFuture {
    context: Arc<Context>,
    //    aspect: EntryAspect,
    id: ProcessUniqueId,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for HoldAspectFuture {
    type Output = Result<(), HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("HoldAspectFuture") {
            return Poll::Ready(Err(err));
        }
        self.context
            .register_waker(self.id.clone(), cx.waker().clone());
        if let Some(state) = self.context.try_state() {
            // wait for the request to complete
            if let Some(result) = state.dht().hold_aspec_request_complete(&self.id) {
                self.context.unregister_waker(self.id.clone());
                Poll::Ready(result.clone())
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
