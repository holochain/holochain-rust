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
use holochain_core_types::{
    cas::content::Address,
    dna::{traits::ReservedTraitNames, Dna},
    entry::{
        cap_entries::{CapTokenGrant, CapabilityType},
        Entry,
    },
    error::HolochainError,
};
use std::{collections::HashMap, pin::Pin, sync::Arc, time::*};

/// Initialization is the value returned by successful initialization of a DNA instance
/// this consists of any public tokens that were granted for use by the container to
/// map any public calls by zome, and an optional payload for the app developer to use as
/// desired
#[derive(Clone, Debug, PartialEq)]
pub struct Initialization {
    public_tokens: HashMap<String, Address>,
    payload: Option<String>,
}

impl Initialization {
    pub fn new() -> Initialization {
        Initialization {
            public_tokens: HashMap::new(),
            payload: None,
        }
    }
    pub fn get_public_token(&self, zome_name: &str) -> Option<Address> {
        self.public_tokens.get(zome_name).map(|addr| addr.clone())
    }
}

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
        return Err(HolochainError::InitializationFailed(
            "Can't trigger initialization: Nucleus status is not New".to_string(),
        ));
    }

    let action_wrapper = ActionWrapper::new(Action::InitApplication(dna.clone()));
    dispatch_action_and_wait(context.clone(), action_wrapper.clone());

    let context_clone = context.clone();

    // internal dispatch ReturnInitializationResult
    fn dispatch_error_result(context: &Arc<Context>, err: HolochainError) {
        context
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnInitializationResult(Err(
                err.to_string(),
            ))))
            .expect("Action channel not usable in initialize_application()");
    }

    // Commit DNA to chain
    let dna_entry = Entry::Dna(dna.clone());
    let dna_commit = await!(commit_entry(dna_entry, None, &context_clone));
    if dna_commit.is_err() {
        dispatch_error_result(&context_clone, dna_commit.err().unwrap());
        return Err(HolochainError::InitializationFailed(
            "error committing DNA".to_string(),
        ));
    }

    // Commit AgentId to chain
    let agent_id_entry = Entry::AgentId(context_clone.agent_id.clone());
    let agent_id_commit = await!(commit_entry(agent_id_entry, None, &context_clone));

    // Let initialization fail if AgentId could not be committed.
    // Currently this cannot happen since ToEntry for Agent always creates
    // an entry from an Agent object. So I can't create a test for the code below.
    // Hence skipping it for codecov for now but leaving it in for resilience.
    if agent_id_commit.is_err() {
        dispatch_error_result(&context_clone, agent_id_commit.err().unwrap());
        return Err(HolochainError::InitializationFailed(
            "error committing Agent".to_string(),
        ));
    }

    let mut public_tokens = HashMap::new();
    // Commit Public Capability Grants to chain
    for (zome_name, zome) in dna.clone().zomes {
        let maybe_public = zome
            .traits
            .iter()
            .find(|(cap_name, _)| *cap_name == ReservedTraitNames::Public.as_str());
        if maybe_public.is_some() {
            let (_, cap) = maybe_public.unwrap();
            let maybe_public_cap_grant_entry =
                CapTokenGrant::create(CapabilityType::Public, None, cap.functions.clone());

            // Let initialization fail if Public Grant could not be committed.
            if maybe_public_cap_grant_entry.is_err() {
                dispatch_error_result(&context_clone, maybe_public_cap_grant_entry.err().unwrap());
                return Err(HolochainError::InitializationFailed(
                    "error creating public cap grant".to_string(),
                ));
            }

            let public_cap_grant_commit = await!(commit_entry(
                Entry::CapTokenGrant(maybe_public_cap_grant_entry.ok().unwrap()),
                None,
                &context_clone
            ));

            // Let initialization fail if Public Grant could not be committed.
            match public_cap_grant_commit {
                Err(err) => {
                    dispatch_error_result(&context_clone, err);
                    return Err(HolochainError::InitializationFailed(
                        "error committing public grant".to_string(),
                    ));
                }
                Ok(addr) => public_tokens.insert(zome_name, addr),
            };
        }
    }

    // map genesis across every zome
    let results: Vec<_> = dna
        .zomes
        .keys()
        .map(|zome_name| genesis(context_clone.clone(), zome_name, &CallbackParams::Genesis))
        .collect();

    // if there was an error report that as the result
    let maybe_error = results
        .iter()
        .find(|ref r| match r {
            CallbackResult::Fail(_) => true,
            _ => false,
        })
        .and_then(|result| match result {
            CallbackResult::Fail(error_string) => Some(error_string.clone()),
            _ => unreachable!(),
        });

    // otherwise return the Initialization struct
    let initialization_result = match maybe_error {
        Some(error_message) => Err(error_message),
        None => Ok(Initialization {
            public_tokens,
            payload: None, // no payload for now
        }),
    };

    context_clone
        .action_channel()
        .send(ActionWrapper::new(Action::ReturnInitializationResult(
            initialization_result,
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
                NucleusStatus::Initialized(ref init) => {
                    Poll::Ready(Ok(NucleusStatus::Initialized(init.clone())))
                }
                NucleusStatus::InitializationFailed(ref error) => {
                    Poll::Ready(Err(HolochainError::ErrorGeneric(error.clone())))
                }
            }
        } else {
            Poll::Pending
        }
    }
}
