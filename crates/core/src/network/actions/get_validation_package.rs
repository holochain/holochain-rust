use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};

use holochain_persistence_api::cas::content::Address;

use holochain_core_types::{
    chain_header::ChainHeader, error::HcResult, validation::ValidationPackage,
};

use std::{pin::Pin, sync::Arc,time::{Instant,Duration}};

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
    GetValidationPackageFuture {
        context: context.clone(),
        address: entry_address,
        running_time:Instant::now()
    }
    .await
}

/// GetValidationPackageFuture resolves to an Option<ValidationPackage>
/// which would be None if the source responded with None, indicating that it
/// is not the source.
pub struct GetValidationPackageFuture {
    context: Arc<Context>,
    address: Address,
    running_time:Instant
}

impl Future for GetValidationPackageFuture {
    type Output = HcResult<Option<ValidationPackage>>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if self.running_time.elapsed() > Duration::from_secs(70)
        {
            panic!("future has been running for too long")
        }
        else
        {
            
        }
        self.context.future_trace.write().expect("Could not get future trace").start_capture("GetValidationPackageFuture".to_string());
        if let Some(err) = self
            .context
            .action_channel_error("GetValidationPackageFuture")
        {
            return Poll::Ready(Err(err));
        }

        if let Some(state) = self.context.try_state() {
            let state = state.network();
            if let Err(error) = state.initialized() {
                return Poll::Ready(Err(error));
            }
            //
            // TODO: connect the waker to state updates for performance reasons
            // See: https://github.com/holochain/holochain-rust/issues/314
            //
            cx.waker().clone().wake();
            self.context.future_trace.write().expect("Could not get future trace").end_capture(String::from("GetValidationPackageFuture"));
            match state.get_validation_package_results.get(&self.address) {
                Some(Some(result)) => Poll::Ready(result.clone()),
                _ => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
