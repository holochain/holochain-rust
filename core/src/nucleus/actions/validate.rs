use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::ribosome::callback::{self, CallbackResult},
};
use futures::{
    future::{self, Future, FutureObj},
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::AddressableContent,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    hash::HashString,
    validation::ValidationData,
};
use snowflake;
use std::{pin::Pin, sync::Arc, thread};

/// ValidateEntry Action Creator
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn validate_entry<'a>(
    entry: Entry,
    validation_data: ValidationData,
    context: &'a Arc<Context>,
) -> FutureObj<'a, Result<HashString, HolochainError>> {
    let id = snowflake::ProcessUniqueId::new();
    let address = entry.address();

    match entry.entry_type() {
        EntryType::App(app_entry_type) => {
            if context
                .state()
                .unwrap()
                .nucleus()
                .dna()
                .unwrap()
                .get_zome_name_for_app_entry_type(&app_entry_type)
                .is_none()
            {
                return FutureObj::new(Box::new(future::err(HolochainError::ValidationFailed(
                    format!(
                        "Attempted to validate unknown app entry type {:?}",
                        app_entry_type,
                    ),
                ))));
            }
        }

        EntryType::LinkAdd => {
            // LinkAdd can always be validated
        }

        EntryType::LinkRemove => {
            // LinkAdd can always be validated
        }

        EntryType::Deletion => {
            // FIXME
        }

        EntryType::CapTokenGrant => {
            // FIXME
        }

        EntryType::AgentId => {
            // FIXME
        }
        _ => {
            return FutureObj::new(Box::new(future::err(HolochainError::ValidationFailed(
                format!(
                    "Attempted to validate system entry type {:?}",
                    entry.entry_type(),
                ),
            ))));
        }
    }

    {
        let id = id.clone();
        let address = address.clone();
        let entry = entry.clone();
        let context = context.clone();
        thread::spawn(move || {
            let maybe_validation_result = callback::validate_entry::validate_entry(
                entry.clone(),
                validation_data.clone(),
                context.clone(),
            );

            let result = match maybe_validation_result {
                Ok(validation_result) => match validation_result {
                    CallbackResult::Fail(error_string) => Err(error_string),
                    CallbackResult::Pass => Ok(()),
                    CallbackResult::NotImplemented(reason) => Err(format!(
                        "Validation callback not implemented for {:?} ({})",
                        entry.entry_type().clone(),
                        reason
                    )),
                    _ => unreachable!(),
                },
                Err(error) => Err(error.to_string()),
            };

            context
                .action_channel()
                .send(ActionWrapper::new(Action::ReturnValidationResult((
                    (id, address),
                    result,
                ))))
                .expect("action channel to be open in reducer");
        });
    };

    FutureObj::new(Box::new(ValidationFuture {
        context: context.clone(),
        key: (id, address),
    }))
}

/// ValidationFuture resolves to an Ok(ActionWrapper) or an Err(error_message:String).
/// Tracks the state for ValidationResults.
pub struct ValidationFuture {
    context: Arc<Context>,
    key: (snowflake::ProcessUniqueId, HashString),
}

impl Future for ValidationFuture {
    type Output = Result<HashString, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
            match state.nucleus().validation_results.get(&self.key) {
                Some(Ok(())) => Poll::Ready(Ok(self.key.1.clone())),
                Some(Err(e)) => Poll::Ready(Err(HolochainError::ValidationFailed(e.clone()))),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
