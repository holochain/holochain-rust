use crate::{
    action::{Action, ActionWrapper, ValidationKey},
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};

use holochain_core_types::{
    chain_header::ChainHeader, error::HcResult, validation::ValidationPackage,
};

use std::{pin::Pin, sync::Arc, thread, time::Duration};

/// GetValidationPackage Action Creator
/// This triggers the network module to retrieve the validation package for the
/// entry given by the header.
///
/// Returns a future that resolves to Option<ValidationPackage> (or HolochainError).
/// If that is None this means that we couldn't get a validation package from the source.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn get_validation_package(
    header: ChainHeader,
    context: &Arc<Context>,
) -> HcResult<Option<ValidationPackage>> {
    let entry_address = header.entry_address().clone();
    let key = ValidationKey {
        address: entry_address,
        id: snowflake::ProcessUniqueId::new().to_string(),
    };
    let action_wrapper = ActionWrapper::new(Action::GetValidationPackage((key.clone(), header)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    GetValidationPackageFuture {
        context: context.clone(),
        key,
    }
    .await
}

/// GetValidationPackageFuture resolves to an Option<ValidationPackage>
/// which would be None if the source responded with None, indicating that it
/// is not the source.
pub struct GetValidationPackageFuture {
    context: Arc<Context>,
    key: ValidationKey,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for GetValidationPackageFuture {
    type Output = HcResult<Option<ValidationPackage>>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self
            .context
            .action_channel_error("GetValidationPackageFuture")
        {
            return Poll::Ready(Err(err));
        }

        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        let waker_clone = cx.waker().clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1));
            waker_clone.wake();
        });

        if let Some(state) = self.context.try_state() {
            let state = state.network();
            if let Err(error) = state.initialized() {
                return Poll::Ready(Err(error));
            }

            match state.get_validation_package_results.get(&self.key) {
                Some(Some(result)) => {
                    dispatch_action(
                        self.context.action_channel(),
                        ActionWrapper::new(Action::ClearValidationPackageResult(self.key.clone())),
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
