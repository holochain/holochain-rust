extern crate futures;
use action::{Action, ActionWrapper};
use context::Context;
use futures::{future, Async, Future};
use hash::HashString;
use hash_table::entry::Entry;
use nucleus::ribosome::callback::{
    validate_commit::validate_commit, CallbackParams, CallbackResult,
};
use snowflake;
use std::{sync::Arc, thread};

/// ValidateEntry Action Creator
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn validate_entry(
    entry: Entry,
    context: &Arc<Context>,
) -> Box<dyn Future<Item = HashString, Error = String>> {
    let id = snowflake::ProcessUniqueId::new();
    let hash = entry.hash();

    match context
        .state()
        .unwrap()
        .nucleus()
        .dna()
        .unwrap()
        .get_zome_name_for_entry_type(entry.clone().entry_type())
    {
        None => {
            return Box::new(future::err(format!(
                "Unknown entry type: '{}'",
                entry.clone().entry_type()
            )));;
        }
        Some(zome_name) => {
            let id = id.clone();
            let hash = hash.clone();
            let entry = entry.clone();
            let context = context.clone();
            thread::spawn(move || {
                let validation_result = match validate_commit(
                    context.clone(),
                    &zome_name,
                    &CallbackParams::ValidateCommit(entry.clone()),
                ) {
                    CallbackResult::Fail(error_string) => Err(error_string),
                    CallbackResult::Pass => Ok(()),
                    CallbackResult::NotImplemented => Err(format!(
                        "Validation callback not implemented for {:?}",
                        entry.entry_type()
                    )),
                };
                context
                    .action_channel
                    .send(ActionWrapper::new(Action::ReturnValidationResult((
                        (id, hash),
                        validation_result,
                    ))))
                    .expect("action channel to be open in reducer");
            });
        }
    };

    Box::new(ValidationFuture {
        context: context.clone(),
        key: (id, hash),
    })
}

/// ValidationFuture resolves to an Ok(ActionWrapper) or an Err(error_message:String).
/// Tracks the state for ValidationResults.
pub struct ValidationFuture {
    context: Arc<Context>,
    key: (snowflake::ProcessUniqueId, HashString),
}

impl Future for ValidationFuture {
    type Item = HashString;
    type Error = String;

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
            match state.nucleus().validation_results.get(&self.key) {
                Some(Ok(())) => Ok(futures::Async::Ready(self.key.1.clone())),
                Some(Err(e)) => Err(e.clone()),
                None => Ok(futures::Async::Pending),
            }
        } else {
            Ok(futures::Async::Pending)
        }
    }
}
