extern crate futures;
extern crate serde_json;
use action::{Action, ActionWrapper};
use context::Context;
use futures::{future, Async, Future};
use holochain_core_types::{
    cas::content::AddressableContent, entry::Entry, entry_type::EntryType, hash::HashString,
};
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
    entry_type: EntryType,
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> Box<dyn Future<Item = HashString, Error = String>> {
    let id = snowflake::ProcessUniqueId::new();
    let address = entry.address();

    match context
        .state()
        .unwrap()
        .nucleus()
        .dna()
        .unwrap()
        .get_zome_name_for_entry_type(entry_type.as_str())
    {
        None => {
            return Box::new(future::err(format!(
                "Unknown entry type: '{}'",
                entry_type.as_str()
            )));;
        }
        Some(_) => {
            let id = id.clone();
            let address = address.clone();
            let entry = entry.clone();
            let context = context.clone();
            thread::spawn(move || {
                let maybe_validation_result = callback::validate_entry::validate_entry(
                    entry.clone(),
                    entry_type.clone(),
                    validation_data.clone(),
                    context.clone(),
                );

                let result = match maybe_validation_result {
                    Ok(validation_result) => match validation_result {
                        CallbackResult::Fail(error_string) => {
                            let error_object: serde_json::Value =
                                serde_json::from_str(&error_string).unwrap();
                            Err(error_object["Err"].to_string())
                        }
                        CallbackResult::Pass => Ok(()),
                        CallbackResult::NotImplemented => Err(format!(
                            "Validation callback not implemented for {:?}",
                            entry_type.clone()
                        )),
                    },
                    Err(error) => Err(error.to_string()),
                };

                context
                    .action_channel
                    .send(ActionWrapper::new(Action::ReturnValidationResult((
                        (id, address),
                        result,
                    ))))
                    .expect("action channel to be open in reducer");
            });
        }
    };

    Box::new(ValidationFuture {
        context: context.clone(),
        key: (id, address),
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
