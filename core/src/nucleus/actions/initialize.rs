extern crate futures;
use action::{Action, ActionWrapper};
use agent::actions::commit::commit_entry;
use context::Context;
use futures::{executor::block_on, future, Async, Future};
use holochain_core_types::entry::ToEntry;
use holochain_dna::Dna;
use instance::dispatch_action_and_wait;
use nucleus::{
    ribosome::callback::{genesis::genesis, CallbackParams, CallbackResult},
    state::NucleusStatus,
};
use std::{sync::Arc, thread, time::*};

/// Timeout in seconds for initialization process.
/// Future will resolve to an error after this duration.
const INITIALIZATION_TIMEOUT: u64 = 30;

/// Initialize Application, Action Creator
/// This is the high-level initialization function that wraps the whole process of initializing an
/// instance. It creates both InitApplication and ReturnInitializationResult actions asynchronously.
///
/// Returns a future that resolves to an Ok(NucleusStatus) or an Err(String) which carries either
/// the Dna error or errors from the genesis callback.
///
/// Use futures::executor::block_on to wait for an initialized instance.
pub fn initialize_application(
    dna: Dna,
    context: Arc<Context>,
) -> Box<dyn Future<Item = NucleusStatus, Error = String>> {
    if context.state().unwrap().nucleus().status != NucleusStatus::New {
        return Box::new(future::err(
            "Can't trigger initialization: Nucleus status is not New".to_string(),
        ));
    }

    let context_clone = context.clone();

    thread::spawn(move || {
        let action_wrapper = ActionWrapper::new(Action::InitApplication(dna.clone()));
        dispatch_action_and_wait(
            &context_clone.action_channel,
            &context_clone.observer_channel,
            action_wrapper.clone(),
        );

        // Commit DNA to chain
        let dna_entry = dna.to_entry();
        let dna_commit = block_on(commit_entry(
            dna_entry,
            &context_clone.action_channel.clone(),
            &context_clone,
        ));

        // Let initialization fail if DNA could not be committed.
        // Currently this cannot happen since ToEntry for Dna always creates
        // an entry from a Dna object. So I can't create a test for the code below.
        // Hence skipping it for codecov for now but leaving it in for resilience.
        #[cfg_attr(tarpaulin, skip)]
        {
            if dna_commit.is_err() {
                context_clone
                    .action_channel
                    .send(ActionWrapper::new(Action::ReturnInitializationResult(
                        Some(dna_commit.err().unwrap()),
                    )))
                    .expect("Action channel not usable in initialize_application()");
                return;
            };
        }

        // map genesis across every zome
        let results: Vec<_> = dna
            .zomes
            .keys()
            .map(|zome_name| genesis(context_clone.clone(), zome_name, &CallbackParams::Genesis))
            .collect();

        let fail_result = results.iter().find(|ref r| match r {
            CallbackResult::Fail(_) => true,
            _ => false,
        });

        let maybe_error = match fail_result {
            Some(result) => match result {
                CallbackResult::Fail(error_string) => Some(error_string.clone()),
                _ => None,
            },
            None => None,
        };

        context_clone
            .action_channel
            .send(ActionWrapper::new(Action::ReturnInitializationResult(
                maybe_error,
            )))
            .expect("Action channel not usable in initialize_application()");
    });

    Box::new(InitializationFuture {
        context: context.clone(),
        created_at: Instant::now(),
    })
}

/// InitializationFuture resolves to an Ok(NucleusStatus) or an Err(String).
/// Tracks the nucleus status.
pub struct InitializationFuture {
    context: Arc<Context>,
    created_at: Instant,
}

impl Future for InitializationFuture {
    type Item = NucleusStatus;
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

        if Instant::now().duration_since(self.created_at)
            > Duration::from_secs(INITIALIZATION_TIMEOUT)
        {
            return Err("Timeout while initializing".to_string());
        }
        if let Some(state) = self.context.state() {
            match state.nucleus().status {
                NucleusStatus::New => Ok(futures::Async::Pending),
                NucleusStatus::Initializing => Ok(futures::Async::Pending),
                NucleusStatus::Initialized => Ok(futures::Async::Ready(NucleusStatus::Initialized)),
                NucleusStatus::InitializationFailed(ref error) => Err(error.clone()),
            }
        } else {
            Ok(futures::Async::Pending)
        }
    }
}
