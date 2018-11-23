extern crate futures;
extern crate serde_json;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{Async, Future};
use holochain_core_types::{error::HolochainError, link::Link};
use std::sync::Arc;

/// AddLink Action Creator
/// This action creator dispatches an AddLink action which is consumed by the DHT reducer.
/// Note that this function does not include any validation checks for the link.
/// The DHT reducer does make sure that it only adds links to a base that it has in its
/// local storage and will return an error that the AddLinkFuture resolves to
/// if that is not the case.
///
/// Returns a future that resolves to an Ok(()) or an Err(HolochainError).
pub fn add_link(link: &Link, context: &Arc<Context>) -> AddLinkFuture {
    let action_wrapper = ActionWrapper::new(Action::AddLink(link.clone()));
    dispatch_action(&context.action_channel, action_wrapper.clone());

    AddLinkFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

pub struct AddLinkFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for AddLinkFuture {
    type Item = ();
    type Error = HolochainError;

    fn poll(
        &mut self,
        cx: &mut futures::task::Context<'_>,
    ) -> Result<Async<Self::Item>, Self::Error> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().wake();
        if let Some(state) = self.context.state() {
            match state.dht().add_link_actions().get(&self.action) {
                Some(Ok(())) => Ok(futures::Async::Ready(())),
                Some(Err(e)) => Err(e.clone()),
                None => Ok(futures::Async::Pending),
            }
        } else {
            Ok(futures::Async::Pending)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleus;

    use futures::executor::block_on;
    use holochain_core_types::{cas::content::AddressableContent, entry::Entry, link::Link};

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry() -> Entry {
        nucleus::actions::tests::test_entry_package_entry()
    }

    #[test]
    fn can_add_valid_link() {
        let (_instance, context) = nucleus::actions::tests::instance();

        let base = test_entry();
        nucleus::actions::tests::commit(base.clone(), &context);

        let target = base.clone();
        let link = Link::new(&base.address(), &target.address(), "test-tag");

        let result = block_on(add_link(&link, &context.clone()));

        assert!(result.is_ok(), "result = {:?}", result);
    }

    #[test]
    fn errors_when_link_base_not_present() {
        let (_instance, context) = nucleus::actions::tests::instance();

        let base = test_entry();
        let target = base.clone();
        let link = Link::new(&base.address(), &target.address(), "test-tag");

        let result = block_on(add_link(&link, &context.clone()));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            HolochainError::ErrorGeneric(String::from("Base for link not found",))
        );
    }
}
