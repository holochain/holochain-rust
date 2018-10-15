extern crate futures;
extern crate serde_json;
use action::{Action, ActionWrapper};
use context::Context;
use futures::{future, Async, Future};
use holochain_core_types::{cas::content::AddressableContent, entry::Entry, hash::HashString};
use holochain_wasm_utils::validation::ValidationData;
use nucleus::ribosome::callback::{self, CallbackResult};
use snowflake;
use std::{sync::Arc, thread};

/// ValidateEntry Action Creator
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn validate_entry(
    entry: &Entry,
    validation_data: &ValidationData,
    context: &Arc<Context>,
) -> Box<dyn Future<Item = HashString, Error = String>> {
    let id = snowflake::ProcessUniqueId::new();

    fn threaded_process_validation(
        id: &snowflake::ProcessUniqueId,
        entry: &Entry,
        context: Arc<Context>,
        validation_data: &ValidationData,
    ) {
        let thread_id = id.clone();
        let thread_entry = entry.clone();
        let thread_context = Arc::clone(&context);
        let thread_validation_data = validation_data.clone();
        let handle = thread::spawn(move || {
            let maybe_validation_result = callback::validate_entry::validate_entry(
                &thread_entry,
                &thread_validation_data,
                thread_context,
            );

            let result = match maybe_validation_result {
                Ok(validation_result) => match validation_result {
                    CallbackResult::Fail(error_string) => Err(error_string),
                    CallbackResult::Pass => Ok(()),
                    CallbackResult::NotImplemented => {
                        Err(format!(
                            "Validation callback not implemented for {:?}",
                            thread_entry.entry_type(),
                        ))
                    }
                },
                Err(error) => Err(error.to_string()),
            };

            context
                .action_channel
                .send(ActionWrapper::new(Action::ReturnValidationResult((
                    (thread_id, thread_entry.address()),
                    result,
                ))))
                .expect("action channel to be open in reducer");
        });
        handle.join().expect("validation thread panicked")
    }

    match entry {
        // app entries validate through zome. lookup said zome.
        Entry::App(app_entry_type, _) => {
            match context
                .state()
                .unwrap()
                .nucleus()
                .dna()
                .unwrap()
                .get_zome_name_for_entry_type(&app_entry_type)
            {
                // failed to lookup zome
                None => {
                    return Box::new(future::err(format!(
                        "Unknown entry type: '{}'",
                        app_entry_type
                    )));
                }
                Some(_) => {
                    threaded_process_validation(&id, &entry, Arc::clone(context), &validation_data);
                }
            };
        }
        // validate system entries
        _ => {
            threaded_process_validation(&id, &entry, Arc::clone(context), &validation_data);
        }
    }

    Box::new(ValidationFuture {
        context: context.clone(),
        key: (id, entry.address()),
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
