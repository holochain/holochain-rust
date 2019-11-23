use crate::{
    action::Action, context::Context, entry::CanPublish,
    network::entry_with_header::EntryWithHeader, nucleus::ZomeFnCall,
};
use holochain_core_types::{entry::Entry, link::link_data::LinkData, network::entry_aspect::EntryAspect};
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
    Publish(Address),                                           // -> Hold
    InitializeNetwork, // -> Hold (the AgentId if initialize chain happend)
    InitializeChain,   // -> prepare to hold AgentId
    SignalZomeFunctionCall(String, snowflake::ProcessUniqueId), // -> ReturnZomeFunctionResult

    // EFFECTS
    Hold(Address),                                                // <- Publish
    UpdateEntry(Address, Address),                                // <- Publish, entry_type=Update
    RemoveEntry(Address, Address),                                // <- Publish, entry_type=Deletion
    AddLink(LinkData),                                            // <- Publish, entry_type=LinkAdd
    RemoveLink(Address),                                            // <- Publish, entry_type=LinkRemove
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
    commit_cache: HashMap<Address, ConsistencySignalE>,

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

    pub fn process_action(&mut self, action: &Action) -> Option<ConsistencySignalE> {
        use ConsistencyEvent::*;
        use ConsistencyGroup::*;
        match action {
            Action::Commit((entry, crud_link, _)) => {
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
                    let hold = Hold(address.clone());
                    let meta = match entry {
                        Entry::App(_, _) => crud_link
                            .clone()
                            .and_then(|crud| Some(UpdateEntry(crud, address.clone()))),
                        Entry::Deletion(_) => crud_link
                            .clone()
                            .and_then(|crud| Some(RemoveEntry(crud, address.clone()))),
                        Entry::LinkAdd(link_data) => Some(AddLink(link_data.clone())),
                        Entry::LinkRemove(_) => Some(RemoveLink(address.clone())),
                        // Question: Why does Entry::LinkAdd take LinkData instead of Link?
                        // as of now, link data contains more information than just the link
                        _ => None,
                    };
                    let mut pending = vec![hold];
                    if let Some(m) = meta {
                        pending.push(m)
                    }
                    let signal = ConsistencySignal::new_pending(
                        Publish(address.clone()),
                        Validators,
                        pending,
                    );
                    self.commit_cache.insert(address, signal);
                }
                None
            }
            Action::Publish(address) => {
                // Emit the signal that was created when observing the corresponding Commit
                let maybe_signal = self.commit_cache.remove(address);
                maybe_signal.or_else(|| {
                    log_warn!(
                        self.context,
                        "consistency: Publishing address that was not previously committed"
                    );
                    None
                })
            }
            Action::HoldAspect(aspect) => match aspect {
                EntryAspect::Content(entry, _) => Some(ConsistencySignal::new_terminal(Hold(entry.address()))),
                EntryAspect::Update(_, header) => {
                    header.link_update_delete().map(|old| {
                        let new = header.entry_address().clone();
                        ConsistencySignal::new_terminal(
                            ConsistencyEvent::UpdateEntry(old, new),
                        )
                    }).or_else(|| {
                        error!("Got header without link_update_delete associated with EntryAspect::Update");
                        None
                    })
                },
                EntryAspect::Deletion(header) => {
                    header.link_update_delete().map(|old| {
                        let new = header.entry_address().clone();
                        ConsistencySignal::new_terminal(
                            ConsistencyEvent::RemoveEntry(old, new),
                        )
                    }).or_else(|| {
                        error!("Got header without link_update_delete associated with EntryAspect::Deletion");
                        None
                    })
                },
                EntryAspect::LinkAdd(data, _) => Some(ConsistencySignal::new_terminal(
                    ConsistencyEvent::AddLink(data.clone()),
                )),
                EntryAspect::LinkRemove(_, header) => Some(ConsistencySignal::new_terminal(
                    ConsistencyEvent::RemoveLink(header.entry_address().clone()),
                )),
            }

            Action::QueueZomeFunctionCall(call) => Some(ConsistencySignal::new_pending(
                SignalZomeFunctionCall(display_zome_fn_call(call), call.id()),
                Source,
                vec![ReturnZomeFunctionResult(
                    display_zome_fn_call(call),
                    call.id(),
                )],
            )),
            Action::ReturnZomeFunctionResult(result) => Some(ConsistencySignal::new_terminal(
                ReturnZomeFunctionResult(display_zome_fn_call(&result.call()), result.call().id()),
            )),
            Action::InitNetwork(settings) => {
                // If the chain was initialized earlier than we also should have
                // committed the agent and so we should be able to wait for the agent id
                // to propagate
                if self.chain_initialized {
                    Some(ConsistencySignal::new_pending(
                        InitializeChain,
                        Validators,
                        vec![Hold(Address::from(settings.agent_id.clone()))],
                    ))
                } else {
                    None
                }
            }
            Action::InitializeChain(_) => {
                self.chain_initialized = true;
                None
            }
            _ => None,
        }
    }
}

fn display_zome_fn_call(call: &ZomeFnCall) -> String {
    format!("{}/{}", call.zome_name, call.fn_name)
}
