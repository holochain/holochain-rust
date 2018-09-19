use hash::HashString;
use hash_table::{
    entry::Entry,
    sys_entry::{EntryType, ToEntry},
};
use serde_json;
use std::str::FromStr;

//-------------------------------------------------------------------------------------------------
// Link
//-------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Link {
    base: HashString,
    target: HashString,
    tag: String,
}

impl Link {
    pub fn new(base: &HashString, target: &HashString, tag: &str) -> Self {
        Link {
            base: base.clone(),
            target: target.clone(),
            tag: tag.to_string(),
        }
    }
    // Key for HashTable
    pub fn key(&self) -> String {
        format!("link:{}:{}:{}", self.base, self.target, self.tag)
    }
    pub fn to_attribute_name(&self) -> String {
        format!("link:{}:{}", self.base, self.tag)
    }
    // Getters
    pub fn base(&self) -> &HashString {
        &self.base
    }
    pub fn target(&self) -> &HashString {
        &self.target
    }
    pub fn tag(&self) -> &String {
        &self.tag
    }
}
//-------------------------------------------------------------------------------------------------
// LinkEntry
//-------------------------------------------------------------------------------------------------

// HC.LinkAction sync with hdk-rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LinkActionKind {
    ADD,
    DELETE,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LinkEntry {
    action_kind: LinkActionKind,
    link: Link,
}

impl LinkEntry {
    pub fn new(
        action_kind: LinkActionKind,
        base: &HashString,
        target: &HashString,
        tag: &str,
    ) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: Link::new(base, target, tag),
        }
    }

    pub fn from_link(action_kind: LinkActionKind, link: &Link) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: link.clone(),
        }
    }
}
impl ToEntry for LinkEntry {
    // Convert a LinkEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        let json_array = serde_json::to_string(self).expect("LinkEntry should serialize");
        Entry::new(EntryType::Link.as_str(), &json_array)
    }

    fn from_entry(entry: &Entry) -> Self {
        assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::Link);
        serde_json::from_str(&entry.content()).expect("entry is not a valid LinkEntry")
    }
}
//-------------------------------------------------------------------------------------------------
// LinkListEntry
//-------------------------------------------------------------------------------------------------
//
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct LinkListEntry {
    pub links: Vec<Link>,
}

impl LinkListEntry {
    pub fn new(links: &[Link]) -> Self {
        LinkListEntry {
            links: links.to_vec(),
        }
    }
}

impl ToEntry for LinkListEntry {
    // Convert a LinkListEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        let json_array = serde_json::to_string(self).expect("LinkListEntry failed to serialize");
        Entry::new(EntryType::LinkList.as_str(), &json_array)
    }

    fn from_entry(entry: &Entry) -> Self {
        assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::LinkList);
        serde_json::from_str(&entry.content()).expect("entry failed converting into LinkListEntry")
    }
}
//-------------------------------------------------------------------------------------------------
// Tests
//-------------------------------------------------------------------------------------------------
#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    use super::*;
    use action::{Action, ActionWrapper};
    use hash_table::sys_entry::{EntryType, ToEntry};
    use instance::{tests::test_context, Instance, Observer};
    use std::{str::FromStr, sync::mpsc::channel};

    pub fn create_test_link() -> Link {
        Link::new(
            &HashString::from("12".to_string()),
            &HashString::from("34".to_string()),
            "fake",
        )
    }

    /// Committing a LinkEntry to source chain should work
    #[test]
    fn can_commit_link() {
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex");
        let link = create_test_link();
        let link_entry = LinkListEntry::new(&[link]);
        let commit_action = ActionWrapper::new(Action::Commit(link_entry.to_entry()));
        // Set up instance and process the action
        let instance = Instance::new();
        let state_observers: Vec<Observer> = Vec::new();
        let (_, rx_observer) = channel::<Observer>();
        instance.process_action(commit_action, state_observers, &rx_observer, &context);
        // Check if LinkEntry is found
        assert_eq!(1, instance.state().history.iter().count());
        instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::Commit(entry) => {
                    assert_eq!(
                        EntryType::from_str(&entry.entry_type()).unwrap(),
                        EntryType::LinkList,
                    );
                    assert_eq!(entry.content(), link_entry.to_entry().content());
                    true
                }
                _ => false,
            });
    }
    /// Committing a LinkListEntry to source chain should work
    #[test]
    fn can_commit_multilink() {
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex");
        let link1 = create_test_link();
        let link2 = Link::new(
            &HashString::from("56".to_string()),
            &HashString::from("78".to_string()),
            "faux",
        );
        let link3 = Link::new(
            &HashString::from("90".to_string()),
            &HashString::from("ab".to_string()),
            "fake",
        );
        let link_entry = LinkListEntry::new(&[link1, link2, link3]);
        let commit_action = ActionWrapper::new(Action::Commit(link_entry.to_entry()));
        println!("commit_multilink: {:?}", commit_action);
        // Set up instance and process the action
        let instance = Instance::new();
        let state_observers: Vec<Observer> = Vec::new();
        let (_, rx_observer) = channel::<Observer>();
        instance.process_action(commit_action, state_observers, &rx_observer, &context);
        // Check if LinkEntry is found
        assert_eq!(1, instance.state().history.iter().count());
        instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::Commit(entry) => {
                    assert_eq!(
                        EntryType::from_str(&entry.entry_type()).unwrap(),
                        EntryType::LinkList,
                    );
                    assert_eq!(entry.content(), link_entry.to_entry().content());
                    true
                }
                _ => false,
            });
    }
    /// Committing a LinkListEntry to source chain should work
    #[test]
    fn can_round_trip_lle() {
        let link = create_test_link();
        let lle = LinkListEntry::new(&[link]);
        let lle_entry = lle.to_entry();
        let lle_trip = LinkListEntry::from_entry(&lle_entry);
        assert_eq!(lle, lle_trip);
    }
}
