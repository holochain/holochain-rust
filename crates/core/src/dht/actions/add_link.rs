use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{error::HolochainError, link::link_data::LinkData};
use std::{pin::Pin, sync::Arc,time::{Instant,Duration}};

/// AddLink Action Creator
/// This action creator dispatches an AddLink action which is consumed by the DHT reducer.
/// Note that this function does not include any validation checks for the link.
/// The DHT reducer does make sure that it only adds links to a base that it has in its
/// local storage and will return an error that the AddLinkFuture resolves to
/// if that is not the case.
///
/// Returns a future that resolves to an Ok(()) or an Err(HolochainError).
pub fn add_link(link: &LinkData, context: &Arc<Context>) -> AddLinkFuture {
    let action_wrapper = ActionWrapper::new(Action::AddLink(link.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    AddLinkFuture {
        context: context.clone(),
        action: action_wrapper,
        running_time:Instant::now()
    }
}

pub struct AddLinkFuture {
    context: Arc<Context>,
    action: ActionWrapper,
    running_time:Instant
}

impl Future for AddLinkFuture {
    type Output = Result<(), HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.context.future_trace.write().expect("Could not get future trace").capture("AddLinkFuture".to_string(),self.running_time.elapsed());
        if let Some(err) = self.context.action_channel_error("AddLinkFuture") {
            return Poll::Ready(Err(err));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        if let Some(state) = self.context.try_state() {
            match state.dht().actions().get(&self.action) {
                Some(Ok(_)) => Poll::Ready(Ok(())),
                Some(Err(e)) => Poll::Ready(Err(e.clone())),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleus;
    use holochain_core_types::{
        agent::test_agent_id,
        chain_header::test_chain_header,
        entry::Entry,
        link::{link_data::LinkData, Link, LinkActionKind},
    };
    use holochain_persistence_api::cas::content::AddressableContent;

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry() -> Entry {
        nucleus::actions::tests::test_entry_package_entry()
    }

    #[test]
    fn can_add_valid_link() {
        let (_instance, context) = nucleus::actions::tests::instance(None);

        let base = test_entry();
        nucleus::actions::tests::commit(base.clone(), &context);

        let target = base.clone();
        let link = Link::new(&base.address(), &target.address(), "test-link", "test-tag");
        let link_data = LinkData::from_link(
            &link,
            LinkActionKind::ADD,
            test_chain_header(),
            test_agent_id(),
        );
        let result = context.block_on(add_link(&link_data, &context.clone()));

        assert!(result.is_ok(), "result = {:?}", result);
    }

    #[test]
    fn errors_when_link_base_not_present() {
        let (_instance, context) = nucleus::actions::tests::instance(None);

        let base = test_entry();
        let target = base.clone();
        let link = Link::new(&base.address(), &target.address(), "test-link", "test-tag");
        let link_data = LinkData::from_link(
            &link,
            LinkActionKind::ADD,
            test_chain_header(),
            test_agent_id(),
        );
        let result = context.block_on(add_link(&link_data, &context.clone()));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            HolochainError::ErrorGeneric(String::from("Base for link not found",))
        );
    }
}
