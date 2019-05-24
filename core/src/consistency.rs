use crate::{action::Action, network::entry_with_header::EntryWithHeader};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::Entry,
    link::Link,
};
use std::collections::HashMap;

pub struct ConsistencyModel {
    commit_cache: HashMap<Address, ConsistencySignal>,
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
    Publish(Address),              // -> Hold
    AddPendingValidation(Address), // -> RemovePendingValidation

    // EFFECTS
    Hold(Address),                    // <- Publish
    UpdateEntry(Address, Address),    // <- Publish, entry_type=Update
    RemoveEntry(Address, Address),    // <- Publish, entry_type=Deletion
    AddLink(Link),                    // <- Publish, entry_type=LinkAdd
    RemoveLink(Link),                 // <- Publish, entry_type=LinkRemove
    RemovePendingValidation(Address), // <- AddPendingValidation
}

#[derive(Clone, Debug, Serialize)]
struct PendingConsistency {
    event: ConsistencyEvent,
    group: ConsistencyGroup,
}

#[derive(Clone, Debug, Serialize)]
pub enum ConsistencyGroup {
    Committer,
    Validators,
}

impl ConsistencyModel {
    pub fn process_action(&mut self, action: &Action) -> Option<ConsistencySignal> {
        use ConsistencyEvent::*;
        use ConsistencyGroup::*;
        match action {
            Action::Commit((entry, crud_link, _)) => {
                let address = entry.address();
                let hold = Hold(address.clone());
                let meta = crud_link.clone().and_then(|crud| match entry {
                    Entry::App(_, _) => Some(UpdateEntry(crud, address.clone())),
                    Entry::Deletion(_) => Some(UpdateEntry(crud, address.clone())),
                    Entry::LinkAdd(link_data) => Some(AddLink(link_data.clone().link)),
                    Entry::LinkRemove(link_data) => Some(RemoveLink(link_data.clone().link)),
                    // Question: Why does Entry::LinkAdd take LinkData instead of Link?
                    _ => None,
                });
                let mut pending = vec![hold];
                meta.map(|m| pending.push(m));
                let signal =
                    ConsistencySignal::new_pending(Publish(address.clone()), Validators, pending);
                self.commit_cache.insert(address, signal);
                None
            }
            Action::Publish(address) => self.commit_cache.remove(address).or_else(|| {
                // TODO: hook up logger
                println!("warn/consistency: Publishing address that was not previously committed");
                None
            }),
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
            Action::RemoveLink(link) => Some(ConsistencySignal::new_terminal(
                ConsistencyEvent::RemoveLink(link.clone()),
            )),

            Action::AddPendingValidation(validation) => {
                let address = validation.entry_with_header.entry.address();
                Some(ConsistencySignal::new_pending(
                    AddPendingValidation(address.clone()),
                    Committer,
                    vec![RemovePendingValidation(address.clone())],
                ))
            }
            Action::RemovePendingValidation((address, _)) => Some(ConsistencySignal::new_terminal(
                RemovePendingValidation(address.clone()),
            )),
            _ => None,
        }
    }
}
