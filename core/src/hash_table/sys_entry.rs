use hash_table::entry::Entry;
use holochain_agent::{Agent, Identity};
use holochain_dna::Dna;
use serde_json;
use std::str::FromStr;

pub trait ToEntry {
    fn to_entry(&self) -> Entry;
    fn from_entry(&Entry) -> Self;
}

//-------------------------------------------------------------------------------------------------
// Entry Type
//-------------------------------------------------------------------------------------------------

// Macro for statically concatanating the system entry prefix for entry types of system entries
macro_rules! sys_prefix {
    ($s:expr) => {
        concat!("%", $s)
    };
}

// Enum for listing all System Entry Types
// Variant `Data` is for user defined entry types
#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    AgentId,
    Deletion,
    App,
    Dna,
    Header,
    Key,
    Link,
    Migration,
    /// TODO #339 - This is different kind of SystemEntry for the DHT only.
    /// Should be moved into a different enum for DHT entry types.
    LinkList,
}

impl FromStr for EntryType {
    type Err = usize;
    // Note: Function always return Ok()
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            sys_prefix!("agent_id") => Ok(EntryType::AgentId),
            sys_prefix!("deletion") => Ok(EntryType::Deletion),
            sys_prefix!("dna") => Ok(EntryType::Dna),
            sys_prefix!("header") => Ok(EntryType::Header),
            sys_prefix!("key") => Ok(EntryType::Key),
            sys_prefix!("link") => Ok(EntryType::Link),
            sys_prefix!("link_list") => Ok(EntryType::LinkList),
            sys_prefix!("migration") => Ok(EntryType::Migration),
            _ => Ok(EntryType::App),
        }
    }
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            EntryType::App => panic!("should not try to convert an app entry type to str"),
            EntryType::AgentId => sys_prefix!("agent_id"),
            EntryType::Deletion => sys_prefix!("deletion"),
            EntryType::Dna => sys_prefix!("dna"),
            EntryType::Header => sys_prefix!("header"),
            EntryType::Key => sys_prefix!("key"),
            EntryType::Link => sys_prefix!("link"),
            EntryType::LinkList => sys_prefix!("link_list"),
            EntryType::Migration => sys_prefix!("migration"),
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Dna Entry
//-------------------------------------------------------------------------------------------------

impl ToEntry for Dna {
    fn to_entry(&self) -> Entry {
        // TODO #239 - Convert Dna to Entry by following DnaEntry schema and not the to_json() dump
        Entry::new(EntryType::Dna.as_str(), &self.to_json())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::Dna);
        return Dna::from_json_str(&entry.value()).expect("entry is not a valid Dna Entry");
    }
}

//-------------------------------------------------------------------------------------------------
// Agent Entry
//-------------------------------------------------------------------------------------------------

impl ToEntry for Agent {
    fn to_entry(&self) -> Entry {
        Entry::new(EntryType::AgentId.as_str(), &self.to_string())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::AgentId);
        let id_content: String =
            serde_json::from_str(&entry.value()).expect("entry is not a valid AgentId Entry");
        Agent::new(Identity::new(id_content))
    }
}

//-------------------------------------------------------------------------------------------------
// UNIT TESTS
//-------------------------------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
    extern crate test_utils;

    use action::{Action, ActionWrapper};
    use hash_table::sys_entry::{EntryType, ToEntry};
    use std::str::FromStr;

    use instance::{tests::test_context, Instance, Observer};
    use std::sync::mpsc::channel;

    /// Committing a DnaEntry to source chain should work
    #[test]
    fn can_commit_dna() {
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex");
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let dna_entry = dna.to_entry();
        let commit_action = ActionWrapper::new(Action::Commit(dna_entry.clone()));

        // Set up instance and process the action
        let instance = Instance::new();
        let state_observers: Vec<Observer> = Vec::new();
        let (_, rx_observer) = channel::<Observer>();
        instance.process_action(commit_action, state_observers, &rx_observer, &context);

        // Check if AgentIdEntry is found
        assert_eq!(1, instance.state().history.iter().count());
        instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::Commit(entry) => {
                    assert_eq!(
                        EntryType::from_str(&entry.entry_type()).unwrap(),
                        EntryType::Dna,
                    );
                    assert_eq!(entry.value(), dna_entry.value());
                    true
                }
                _ => false,
            });
    }

    /// Committing an AgentIdEntry to source chain should work
    #[test]
    fn can_commit_agent() {
        // Create Context, Agent and Commit AgentIdEntry Action
        let context = test_context("alex");
        let agent_entry = context.agent.to_entry();
        let commit_agent_action = ActionWrapper::new(Action::Commit(agent_entry.clone()));

        // Set up instance and process the action
        let instance = Instance::new();
        let state_observers: Vec<Observer> = Vec::new();
        let (_, rx_observer) = channel::<Observer>();
        instance.process_action(commit_agent_action, state_observers, &rx_observer, &context);

        // Check if AgentIdEntry is found
        assert_eq!(1, instance.state().history.iter().count());
        instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::Commit(entry) => {
                    assert_eq!(
                        EntryType::from_str(&entry.entry_type()).unwrap(),
                        EntryType::AgentId,
                    );
                    assert_eq!(entry.value(), agent_entry.value());
                    true
                }
                _ => false,
            });
    }

    #[test]
    /// converting a str to an EntryType and back
    fn test_from_as_str() {
        for (type_str, variant) in vec![
            (sys_prefix!("agent_id"), EntryType::AgentId),
            (sys_prefix!("deletion"), EntryType::Deletion),
            (sys_prefix!("dna"), EntryType::Dna),
            (sys_prefix!("header"), EntryType::Header),
            (sys_prefix!("key"), EntryType::Key),
            (sys_prefix!("link"), EntryType::Link),
            (sys_prefix!("migration"), EntryType::Migration),
        ] {
            assert_eq!(
                variant,
                EntryType::from_str(type_str).expect("could not convert str to EntryType")
            );

            assert_eq!(type_str, variant.as_str(),);
        }
    }

}
