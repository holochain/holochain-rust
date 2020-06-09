use crate::{
    action::{Action, ActionWrapper},
    agent::{
        actions::commit::commit_entry, find_chain_header,
        state::create_entry_with_header_for_header,
    },
    context::Context,
    dht::actions::hold_aspect::hold_aspect_no_ack,
    network::entry_aspect::EntryAspect,
    nucleus::state::NucleusStatus,
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{
    dna::{traits::ReservedTraitNames, Dna},
    entry::{
        cap_entries::{CapFunctions, CapTokenGrant, CapabilityType, ReservedCapabilityId},
        Entry,
    },
    error::HolochainError,
};
use holochain_persistence_api::cas::content::Address;

use crate::instance::dispatch_action;
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc, time::*};

/// Initialization is the value returned by successful initialization of a DNA instance
/// this consists of any public tokens that were granted for use by the container to
/// map any public calls by zome, and an optional payload for the app developer to use as
/// desired
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct Initialization {
    public_token: Option<Address>,
    payload: Option<String>,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Initialization {
    pub fn new() -> Initialization {
        Initialization::default()
    }
    pub fn public_token(&self) -> Option<Address> {
        self.public_token.clone()
    }
}

/// Timeout in seconds for initialization process.
/// Future will resolve to an error after this duration.
const INITIALIZATION_TIMEOUT: u64 = 60;

/// Initialize Chain, Action Creator
/// This is the high-level initialization function that wraps the whole process of initializing an
/// instance. It creates both InitializeChain and ReturnInitializationResult actions asynchronously.
///
/// Returns a future that resolves to an Ok(NucleusStatus) or an Err(String) which carries either
/// the Dna error or errors from the init callback.
///
/// Use futures::executor::block_on to wait for an initialized instance.
#[autotrace]
//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn initialize_chain(
    dna: Dna,
    context: &Arc<Context>,
) -> Result<NucleusStatus, HolochainError> {
    if context.state().unwrap().nucleus().status != NucleusStatus::New {
        return Err(HolochainError::InitializationFailed(
            "Can't trigger initialization: Nucleus status is not New".to_string(),
        ));
    }

    let action_wrapper = ActionWrapper::new(Action::InitializeChain(dna.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    let id = ProcessUniqueId::new();
    let _ = InitializingFuture {
        context: context.clone(),
        id,
    }
    .await;

    let context_clone = context.clone();

    // internal dispatch ReturnInitializationResult
    fn dispatch_error_result(context: &Arc<Context>, err: HolochainError) {
        context
            .action_channel()
            .send_wrapped(ActionWrapper::new(Action::ReturnInitializationResult(Err(
                err.to_string(),
            ))))
            .expect("Action channel not usable in initialize_chain()");
    }

    // Commit DNA to chain
    let dna_entry = Entry::Dna(Box::new(dna.clone()));
    let dna_commit = commit_entry(dna_entry.clone(), None, &context_clone).await;
    if dna_commit.is_err() {
        let error = dna_commit.err().unwrap();
        dispatch_error_result(&context_clone, error.clone());
        return Err(HolochainError::InitializationFailed(format!(
            "Error committing DNA: {:?}",
            error
        )));
    }

    // mark the dna header as held.
    let dna_header = find_chain_header(&dna_entry, &context.state().unwrap())
        .ok_or_else(|| HolochainError::from("No header found for dna entry"))?;

    let ewh = create_entry_with_header_for_header(&context.state().unwrap(), dna_header)?;
    let entry_aspect = EntryAspect::Content(ewh.entry, ewh.header);
    hold_aspect_no_ack(&ProcessUniqueId::new(), entry_aspect, context.clone()).await?;

    // Commit AgentId to chain
    let agent_id_entry = Entry::AgentId(context_clone.agent_id.clone());
    let agent_id_commit = commit_entry(agent_id_entry.clone(), None, &context_clone).await;

    // Let initialization fail if AgentId could not be committed.
    // Currently this cannot happen since ToEntry for Agent always creates
    // an entry from an Agent object. So I can't create a test for the code below.
    // Hence skipping it for codecov for now but leaving it in for resilience.
    if agent_id_commit.is_err() {
        dispatch_error_result(&context_clone, agent_id_commit.err().unwrap());
        return Err(HolochainError::InitializationFailed(
            "error committing Agent".to_string(),
        ));
    } else {
        let agent_id_header = find_chain_header(&agent_id_entry, &context.state().unwrap())
            .ok_or_else(|| HolochainError::from("No header found for agent id entry"))?;

        // mark the entry and it's header as held in the dht store because we always hold ourselves.
        let entry_aspect = EntryAspect::Content(agent_id_entry, agent_id_header.clone());
        hold_aspect_no_ack(&ProcessUniqueId::new(), entry_aspect, context.clone()).await?;

        let ewh = create_entry_with_header_for_header(&context.state().unwrap(), agent_id_header)?;
        let entry_aspect = EntryAspect::Content(ewh.entry, ewh.header);
        hold_aspect_no_ack(&ProcessUniqueId::new(), entry_aspect, context.clone()).await?;
    }

    let mut cap_functions = CapFunctions::new();
    let zomes = dna.clone().zomes;
    if zomes.is_empty() {
        return Err(HolochainError::ErrorGeneric(
            "Attempting to initialize DNA with zero zomes!".into(),
        ));
    }
    // Commit Public Capability Grants to chain
    for (zome_name, zome) in zomes {
        let maybe_public = zome
            .traits
            .iter()
            .find(|(cap_name, _)| *cap_name == ReservedTraitNames::Public.as_str());
        if let Some(public) = maybe_public {
            let (_, cap) = public;
            cap_functions.insert(zome_name, cap.functions.clone());
        }
    }
    let public_token = if !cap_functions.is_empty() {
        let maybe_public_cap_grant_entry = CapTokenGrant::create(
            ReservedCapabilityId::Public.as_str(),
            CapabilityType::Public,
            None,
            cap_functions,
        );

        // Let initialization fail if Public Grant could not be created.
        if maybe_public_cap_grant_entry.is_err() {
            dispatch_error_result(&context_clone, maybe_public_cap_grant_entry.err().unwrap());
            return Err(HolochainError::InitializationFailed(
                "error creating public cap grant".to_string(),
            ));
        }

        let grant = maybe_public_cap_grant_entry.ok().unwrap();
        let public_cap_grant_commit =
            commit_entry(Entry::CapTokenGrant(grant.clone()), None, &context_clone).await;

        // Let initialization fail if Public Grant could not be committed.
        match public_cap_grant_commit {
            Err(err) => {
                dispatch_error_result(&context_clone, err);
                return Err(HolochainError::InitializationFailed(
                    "error committing public grant".to_string(),
                ));
            }
            Ok(addr) => {
                log_debug!(context, "initialize: created public token: {:?}", addr);
                Some(addr)
            }
        }
    } else {
        None
    };

    // Note: The calling of the zome init callbacks has been moved to its own action `call_init`
    // This is now called by the initialize workflow in application.rs

    // otherwise return the Initialization struct
    let initialization_result = Ok(Initialization {
        public_token,
        payload: None, // no payload for now
    });

    context_clone
        .action_channel()
        .send_wrapped(ActionWrapper::new(Action::ReturnInitializationResult(
            initialization_result,
        )))
        .expect("Action channel not usable in initialize_chain()");

    let id = ProcessUniqueId::new();
    InitializationFuture {
        context: context.clone(),
        created_at: Instant::now(),
        id,
    }
    .await
}

/// Tracks if the initialization has started and the DNA is set in the nucleus.
pub struct InitializingFuture {
    context: Arc<Context>,
    id: ProcessUniqueId,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for InitializingFuture {
    type Output = Result<NucleusStatus, HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("InitializingFuture") {
            return Poll::Ready(Err(err));
        }
        self.context
            .register_waker(self.id.clone(), cx.waker().clone());

        if let Some(state) = self.context.try_state() {
            match state.nucleus().status {
                NucleusStatus::New => Poll::Pending,
                NucleusStatus::Initializing => {
                    self.context.unregister_waker(self.id.clone());
                    Poll::Ready(Ok(NucleusStatus::Initializing))
                }
                NucleusStatus::Initialized(ref init) => {
                    self.context.unregister_waker(self.id.clone());
                    Poll::Ready(Ok(NucleusStatus::Initialized(init.clone())))
                }
                NucleusStatus::InitializationFailed(ref error) => {
                    self.context.unregister_waker(self.id.clone());
                    Poll::Ready(Err(HolochainError::ErrorGeneric(error.clone())))
                }
            }
        } else {
            Poll::Pending
        }
    }
}

/// InitializationFuture resolves to an Ok(NucleusStatus) or an Err(String).
/// Tracks the nucleus status.
pub struct InitializationFuture {
    context: Arc<Context>,
    created_at: Instant,
    id: ProcessUniqueId,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for InitializationFuture {
    type Output = Result<NucleusStatus, HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("InitializationFuture") {
            return Poll::Ready(Err(err));
        }
        self.context
            .register_waker(self.id.clone(), cx.waker().clone());

        if Instant::now().duration_since(self.created_at)
            > Duration::from_secs(INITIALIZATION_TIMEOUT)
        {
            return Poll::Ready(Err(HolochainError::ErrorGeneric(
                "Timeout while initializing".to_string(),
            )));
        }
        if let Some(state) = self.context.try_state() {
            match state.nucleus().status {
                NucleusStatus::New => Poll::Pending,
                NucleusStatus::Initializing => Poll::Pending,
                NucleusStatus::Initialized(ref init) => {
                    self.context.unregister_waker(self.id.clone());
                    Poll::Ready(Ok(NucleusStatus::Initialized(init.clone())))
                }
                NucleusStatus::InitializationFailed(ref error) => {
                    self.context.unregister_waker(self.id.clone());
                    Poll::Ready(Err(HolochainError::ErrorGeneric(error.clone())))
                }
            }
        } else {
            Poll::Pending
        }
    }
}
