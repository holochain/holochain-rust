extern crate futures;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::Address, chain_header::ChainHeader, error::HcResult,
    validation::ValidationPackage,
};
use std::{
    pin::{Pin, Unpin},
    sync::Arc,
};

/// GetValidationPackage Action Creator
/// This triggers the network module to retrieve the validation package for the
/// entry given by the header.
///
/// Returns a future that resolves to Option<ValidationPackage> (or HolochainError).
/// If that is None this means that we couldn't get a validation package from the source.
pub async fn get_validation_package(
    header: ChainHeader,
    context: &Arc<Context>,
) -> HcResult<Option<ValidationPackage>> {
    let entry_address = header.entry_address().clone();
    let action_wrapper = ActionWrapper::new(Action::GetValidationPackage(header));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    await!(GetValidationPackageFuture {
        context: context.clone(),
        address: entry_address,
    })
}

/// GetValidationPackageFuture resolves to an Option<ValidationPackage>
/// which would be None if the source responded with None, indicating that it
/// is not the source.
pub struct GetValidationPackageFuture {
    context: Arc<Context>,
    address: Address,
}

impl Unpin for GetValidationPackageFuture {}

impl Future for GetValidationPackageFuture {
    type Output = HcResult<Option<ValidationPackage>>;

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
        match state.get_validation_package_results.get(&self.address) {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
