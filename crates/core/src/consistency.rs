use crate::{
    action::Action,
    context::Context,
    entry::CanPublish,
    network::handler::{get_content_aspect, lists::get_base_address_and_meta_aspect},
    nucleus::ZomeFnCall,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug, Serialize)]
pub struct ConsistencySignal<E: Serialize> {
    event: E,
    pending: Vec<PendingConsistency<E>>,
}

impl<E: Serialize> ConsistencySignal<E> {
    pub fn new_terminal(event: E) -> Self {
        Self {
            event,
            pending: Vec::new(),
        }
    }

    pub fn new_pending(event: E, group: ConsistencyGroup, pending_events: Vec<E>) -> Self {
        let pending = pending_events
            .into_iter()
            .map(|event| PendingConsistency {
                event,
                group: group.clone(),
            })
            .collect();
        Self { event, pending }
    }
}

impl From<ConsistencySignalE> for ConsistencySignal<String> {
    fn from(signal: ConsistencySignalE) -> ConsistencySignal<String> {
        let ConsistencySignalE { event, pending } = signal;
        ConsistencySignal {
            event: serde_json::to_string(&event)
                .expect("ConsistencySignal serialization cannot fail"),
            pending: pending
                .into_iter()
                .map(|p| PendingConsistency {
                    event: serde_json::to_string(&p.event)
                        .expect("ConsistencySignal serialization cannot fail"),
                    group: p.group,
                })
                .collect(),
        }
    }
}

type ConsistencySignalE = ConsistencySignal<ConsistencyEvent>;

#[derive(Clone, Debug, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum ConsistencyEvent {
    // CAUSES
    PublishAspect(Address, Address), // -> Hold
    InitializeNetwork,               // -> Hold (the AgentId if initialize chain happend)
    InitializeChain,                 // -> prepare to hold AgentId
    SignalZomeFunctionCall(String, snowflake::ProcessUniqueId), // -> ReturnZomeFunctionResult

    // EFFECTS
    HoldAspect(Address, Address), // <- Publish
    ReturnZomeFunctionResult(String, snowflake::ProcessUniqueId), // <- SignalZomeFunctionCall
}

#[derive(Clone, Debug, Serialize)]
struct PendingConsistency<E: Serialize> {
    event: E,
    group: ConsistencyGroup,
}

#[derive(Clone, Debug, Serialize)]
pub enum ConsistencyGroup {
    Source,
    Validators,
}

#[derive(Clone)]
pub struct ConsistencyModel {
    // upon Commit, caches the corresponding ConsistencySignal which will only be emitted
    // later, when the corresponding Publish has been processed
    commit_cache: HashMap<Address, Vec<ConsistencySignalE>>,

    // store whether we have initialized the chain
    chain_initialized: bool,

    // Context needed to examine state and do logging
    context: Arc<Context>,
}

impl ConsistencyModel {
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            commit_cache: HashMap::new(),
            chain_initialized: false,
            context,
        }
    }

    pub fn process_action(&mut self, action: &Action) -> Vec<ConsistencySignalE> {
        use ConsistencyEvent::*;
        use ConsistencyGroup::*;
        match action {
            Action::Commit((entry, _crud_link, _)) => {
                // XXX: Since can_publish relies on a properly initialized Context, there are a few ways
                // can_publish can fail. If we hit the possiblity of failure, just add the commit to the cache
                // anyway. The only reason to check is to avoid filling up the cache unnecessarily with
                // commits that will never be published.
                let do_cache = self.context.state().is_none()
                    || self.context.get_dna().is_none()
                    || entry.entry_type().can_publish(&self.context);

                // If entry is publishable, construct the ConsistencySignal that should be emitted
                // when the entry is finally published, and save it for later
                if do_cache {
                    let address = entry.address();
                    let content_aspect = get_content_aspect(&address, &self.context)
                        .expect("Must be able to get content aspect for own entry");
                    let header = content_aspect.header();
                    let mut signals = vec![];
                    signals.push(ConsistencySignal::new_pending(
                        PublishAspect(
                            content_aspect.entry_address().clone(),
                            content_aspect.address(),
                        ),
                        Validators,
                        vec![HoldAspect(
                            content_aspect.entry_address().clone(),
                            content_aspect.address(),
                        )],
                    ));
                    if let Some((_, meta_aspect)) =
                        get_base_address_and_meta_aspect(entry.clone(), header.clone())
                    {
                        signals.push(ConsistencySignal::new_pending(
                            PublishAspect(
                                meta_aspect.entry_address().clone(),
                                meta_aspect.address(),
                            ),
                            Validators,
                            vec![HoldAspect(
                                meta_aspect.entry_address().clone(),
                                meta_aspect.address(),
                            )],
                        ));
                    }

                    self.commit_cache.insert(address, signals);
                }
                vec![]
            }
            Action::Publish(address) => {
                // Emit the signal that was created when observing the corresponding Commit
                let maybe_signals = self.commit_cache.remove(address);
                maybe_signals.unwrap_or_else(|| {
                    log_warn!(
                        self.context,
                        "consistency: Publishing address that was not previously committed"
                    );
                    vec![]
                })
            }

            // TODO: how to deal with header publishing, in terms of aspects?
            // Action::PublishHeaderEntry(address) => {
            //     if let Some(header) = self.context.state().unwrap().get_most_recent_header_for_entry_address(address);
            //     vec![ConsistencySignal::new_pending(PublishHeader(address.clone()), Validators, vec![HoldHeader(address.clone())])]
            // }
            Action::HoldAspect(aspect) => vec![ConsistencySignal::new_terminal(HoldAspect(
                aspect.entry_address().clone(),
                aspect.address(),
            ))],

            Action::QueueZomeFunctionCall(call) => vec![ConsistencySignal::new_pending(
                SignalZomeFunctionCall(display_zome_fn_call(call), call.id()),
                Source,
                vec![ReturnZomeFunctionResult(
                    display_zome_fn_call(call),
                    call.id(),
                )],
            )],
            Action::ReturnZomeFunctionResult(result) => vec![ConsistencySignal::new_terminal(
                ReturnZomeFunctionResult(display_zome_fn_call(&result.call()), result.call().id()),
            )],
            // Action::InitNetwork(settings) => {
            //     // If the chain was initialized earlier than we also should have
            //     // committed the agent and so we should be able to wait for the agent id
            //     // to propagate
            //     if self.chain_initialized {
            //         vec!(ConsistencySignal::new_pending(
            //             InitializeChain,
            //             Validators,
            //             vec![Hold(Address::from(settings.agent_id.clone()))],
            //         ))
            //     } else {
            //         vec![]
            //     }
            // }
            // Action::InitializeChain(_) => {
            //     self.chain_initialized = true;
            //     vec![]
            // }
            _ => vec![],
        }
    }
}

fn display_zome_fn_call(call: &ZomeFnCall) -> String {
    format!("{}/{}", call.zome_name, call.fn_name)
}
