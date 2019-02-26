use crate::{
    action::{Action, ActionWrapper},
    agent::actions::commit::commit_entry,
    context::Context,
    instance::dispatch_action_and_wait,
    nucleus::{
        ribosome::callback::{genesis::genesis, CallbackParams, CallbackResult},
        state::NucleusStatus,
    },
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{dna::Dna, entry::Entry, error::HolochainError};
use std::{pin::Pin, sync::Arc, time::*};

/// Timeout in seconds for initialization process.
/// Future will resolve to an error after this duration.
const INITIALIZATION_TIMEOUT: u64 = 60;

/// Initialize Application, Action Creator
/// This is the high-level initialization function that wraps the whole process of initializing an
/// instance. It creates both InitApplication and ReturnInitializationResult actions asynchronously.
///
/// Returns a future that resolves to an Ok(NucleusStatus) or an Err(String) which carries either
/// the Dna error or errors from the genesis callback.
///
/// Use futures::executor::block_on to wait for an initialized instance.
pub async fn initialize_application(
    dna: Dna,
    context: &Arc<Context>,
) -> Result<NucleusStatus, HolochainError> {
    if context.state().unwrap().nucleus().status != NucleusStatus::New {
        return Err(HolochainError::new(
            "Can't trigger initialization: Nucleus status is not New",
        ));
    }

    let action_wrapper = ActionWrapper::new(Action::InitApplication(dna.clone()));
    dispatch_action_and_wait(context.clone(), action_wrapper.clone());

    let context_clone = context.clone();

    // Commit DNA to chain
    let dna_entry = Entry::Dna(dna.clone());
    let dna_commit = await!(commit_entry(dna_entry, None, &context_clone));
    if dna_commit.is_err() {
        // Let initialization fail if DNA could not be committed.
        // Currently this cannot happen since ToEntry for Dna always creates
        // an entry from a Dna object. So I can't create a test for the code below.
        // Hence skipping it for codecov for now but leaving it in for resilience.
        context_clone
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnInitializationResult(
                Some(dna_commit.map_err(|e| e.to_string()).err().unwrap()),
            )))
            .expect("Action channel not usable in initialize_application()");
        return Err(HolochainError::new("error committing DNA"));
    }

    // Commit AgentId to chain
    let agent_id_entry = Entry::AgentId(context_clone.agent_id.clone());
    let agent_id_commit = await!(commit_entry(agent_id_entry, None, &context_clone));

    // Let initialization fail if AgentId could not be committed.
    // Currently this cannot happen since ToEntry for Agent always creates
    // an entry from an Agent object. So I can't create a test for the code below.
    // Hence skipping it for codecov for now but leaving it in for resilience.

    if agent_id_commit.is_err() {
        context_clone
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnInitializationResult(
                Some(agent_id_commit.map_err(|e| e.to_string()).err().unwrap()),
            )))
            .expect("Action channel not usable in initialize_application()");
        return Err(HolochainError::new("error committing Agent"));
    }

    // map genesis across every zome
    let results: Vec<_> = dna
        .zomes
        .keys()
        .map(|zome_name| genesis(context_clone.clone(), zome_name, &CallbackParams::Genesis))
        .collect();

    let maybe_error = results
        .iter()
        .find(|ref r| match r {
            CallbackResult::Fail(_) => true,
            _ => false,
        })
        .and_then(|result| match result {
            CallbackResult::Fail(error_string) => Some(error_string.clone()),
            _ => None,
        });

    context_clone
        .action_channel()
        .send(ActionWrapper::new(Action::ReturnInitializationResult(
            maybe_error,
        )))
        .expect("Action channel not usable in initialize_application()");

    await!(InitializationFuture {
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
    type Output = Result<NucleusStatus, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();

        if Instant::now().duration_since(self.created_at)
            > Duration::from_secs(INITIALIZATION_TIMEOUT)
        {
            return Poll::Ready(Err(HolochainError::ErrorGeneric(
                "Timeout while initializing".to_string(),
            )));
        }
        if let Some(state) = self.context.state() {
            match state.nucleus().status {
                NucleusStatus::New => Poll::Pending,
                NucleusStatus::Initializing => Poll::Pending,
                NucleusStatus::Initialized => Poll::Ready(Ok(NucleusStatus::Initialized)),
                NucleusStatus::InitializationFailed(ref error) => {
                    Poll::Ready(Err(HolochainError::ErrorGeneric(error.clone())))
                }
            }
        } else {
            Poll::Pending
        }
    }
}
