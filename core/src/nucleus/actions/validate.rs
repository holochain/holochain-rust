extern crate futures;
use action::{Action, ActionWrapper};
use context::Context;
use futures::{future, Async, Future};
use hash_table::entry::Entry;
use std::{sync::Arc, thread};
use nucleus::{
    ribosome::callback::{validate_commit::validate_commit, CallbackParams, CallbackResult},
};

/// ValidateEntry Action Creator
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn validate_entry(entry: Entry, context: &Arc<Context>) -> Box<dyn Future<Item = ActionWrapper, Error = String>> {
    let action_wrapper = ActionWrapper::new(Action::ValidateEntry(entry.clone()));
    //dispatch_action(&context.action_channel, action_wrapper.clone());

    match context.state()
        .unwrap()
        .nucleus()
        .dna()
        .unwrap()
        .get_zome_name_for_entry_type(entry.clone().entry_type())
        {
            None => {
                return Box::new(future::err(
                    format!("Unknown entry type: '{}'", entry.clone().entry_type()),
                ));;
            }
            Some(zome_name) => {
                //#[cfg(debug)] state.validations_running.push(action_wrapper.clone());
                let action_wrapper = action_wrapper.clone();
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
                            Box::new(action_wrapper),
                            validation_result,
                        ))))
                        .expect("action channel to be open in reducer");
                });
            }
        };



    Box::new(ValidationFuture {
        context: context.clone(),
        action: action_wrapper,
    })
}

/// ValidationFuture resolves to an Ok(ActionWrapper) or an Err(error_message:String).
/// Tracks the state for ValidationResults.
pub struct ValidationFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for ValidationFuture {
    type Item = ActionWrapper;
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
            match state.nucleus().validation_result(&self.action) {
                Some(Ok(())) => Ok(futures::Async::Ready(self.action.clone())),
                Some(Err(e)) => Err(e),
                None => Ok(futures::Async::Pending),
            }
        } else {
            Ok(futures::Async::Pending)
        }
    }
}
