use crate::{
    action::Action, context::Context, entry::CanPublish,
    network::entry_with_header::EntryWithHeader,
};
use holochain_core_types::{entry::Entry, link::link_data::LinkData};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct ConsistencyModel {
    commit_cache: HashMap<Address, ConsistencySignal>,
    context: Arc<Context>,
}

impl ConsistencyModel {
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            commit_cache: HashMap::new(),
            context,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ConsistencySignal {
    event: ConsistencyEvent,
    pending: Vec<PendingConsistency>,
}

impl ConsistencySignal {
    pub fn new_terminal(event: ConsistencyEvent) -> Self {
        Self {
            event,
            pending: Vec::new(),
        }
    }

    pub fn new_pending(
        event: ConsistencyEvent,
        group: ConsistencyGroup,
        pending_events: Vec<ConsistencyEvent>,
    ) -> Self {
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

#[derive(Clone, Debug, Serialize)]
pub enum ConsistencyEvent {
    // CAUSES
    Publish(Address),                                   // -> Hold
    AddPendingValidation(Address),                      // -> RemovePendingValidation
    SignalZomeFunctionCall(snowflake::ProcessUniqueId), // -> ReturnZomeFunctionResult

    // EFFECTS
    Hold(Address),                                        // <- Publish
    UpdateEntry(Address, Address),                        // <- Publish, entry_type=Update
    RemoveEntry(Address, Address),                        // <- Publish, entry_type=Deletion
    AddLink(LinkData),                                    // <- Publish, entry_type=LinkAdd
    RemoveLink(Entry),                                    // <- Publish, entry_type=LinkRemove
    RemovePendingValidation(Address),                     // <- AddPendingValidation
    ReturnZomeFunctionResult(snowflake::ProcessUniqueId), // <- SignalZomeFunctionCall
}

#[derive(Clone, Debug, Serialize)]
struct PendingConsistency {
    event: ConsistencyEvent,
    group: ConsistencyGroup,
}

#[derive(Clone, Debug, Serialize)]
pub enum ConsistencyGroup {
    Source,
    Validators,
}

impl ConsistencyModel {
    pub fn process_action(&mut self, action: &Action) -> Option<ConsistencySignal> {
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
                    let meta = crud_link.clone().and_then(|crud| match entry {
                        Entry::App(_, _) => Some(UpdateEntry(crud, address.clone())),
                        Entry::Deletion(_) => Some(RemoveEntry(crud, address.clone())),
                        Entry::LinkAdd(link_data) => Some(AddLink(link_data.clone())),
                        Entry::LinkRemove(_) => Some(RemoveLink(entry.clone())),
                        // Question: Why does Entry::LinkAdd take LinkData instead of Link?
                        // as of now, link data contains more information than just the link
                        _ => None,
                    });
                    let mut pending = vec![hold];
                    meta.map(|m| pending.push(m));
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
                    // TODO: hook up logger
                    println!(
                        "warn/consistency: Publishing address that was not previously committed"
                    );
                    None
                })
            }
            Action::Hold(EntryWithHeader { entry, header: _ }) => {
                Some(ConsistencySignal::new_terminal(Hold(entry.address())))
            }
            Action::UpdateEntry((old, new)) => Some(ConsistencySignal::new_terminal(
                ConsistencyEvent::UpdateEntry(old.clone(), new.clone()),
            )),
            Action::RemoveEntry((old, new)) => Some(ConsistencySignal::new_terminal(
                ConsistencyEvent::RemoveEntry(old.clone(), new.clone()),
            )),
            Action::AddLink(link) => Some(ConsistencySignal::new_terminal(
                ConsistencyEvent::AddLink(link.clone()),
            )),
            Action::RemoveLink(entry) => Some(ConsistencySignal::new_terminal(
                ConsistencyEvent::RemoveLink(entry.clone()),
            )),

            Action::AddPendingValidation(validation) => {
                let address = validation.entry_with_header.entry.address();
                Some(ConsistencySignal::new_pending(
                    AddPendingValidation(address.clone()),
                    Source,
                    vec![RemovePendingValidation(address.clone())],
                ))
            }
            Action::RemovePendingValidation((address, _)) => Some(ConsistencySignal::new_terminal(
                RemovePendingValidation(address.clone()),
            )),

            Action::SignalZomeFunctionCall(call) => Some(ConsistencySignal::new_pending(
                SignalZomeFunctionCall(call.id()),
                Source,
                vec![ReturnZomeFunctionResult(call.id())],
            )),
            Action::ReturnZomeFunctionResult(result) => Some(ConsistencySignal::new_terminal(
                ReturnZomeFunctionResult(result.call().id()),
            )),
            _ => None,
        }
    }
}
