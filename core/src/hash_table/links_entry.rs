use cas::content::Address;
use hash_table::{
    entry::Entry,
    sys_entry::{EntryType, ToEntry},
    HashString,
};
use serde_json;

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
    pub fn new(base: &Address, target: &Address, tag: &str) -> Self {
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
    pub fn base(&self) -> &Address {
        &self.base
    }
    pub fn target(&self) -> &Address {
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
    fn to_entry(&self) -> (EntryType, Entry) {
        let json_array = serde_json::to_string(self).expect("LinkEntry should serialize");
        (EntryType::Link, Entry::from(json_array))
    }

    fn from_entry(entry: &Entry) -> Self {
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
    fn to_entry(&self) -> (EntryType, Entry) {
        let json_array = serde_json::to_string(self).expect("LinkListEntry failed to serialize");
        (EntryType::LinkList, Entry::from(json_array))
    }

    fn from_entry(entry: &Entry) -> Self {
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
    use std::sync::mpsc::channel;

    pub fn create_test_link() -> Link {
        Link::new(
            &HashString::from("12".to_string()),
            &HashString::from("34".to_string()),
            "fake",
        )
    }

    pub fn create_test_link_a() -> Link {
        create_test_link()
    }

    pub fn create_test_link_b() -> Link {
        Link::new(
            &HashString::from("56".to_string()),
            &HashString::from("78".to_string()),
            "faux",
        )
    }

    pub fn create_test_link_c() -> Link {
        Link::new(
            &HashString::from("90".to_string()),
            &HashString::from("ab".to_string()),
            "fake",
        )
    }

    /// Committing a LinkEntry to source chain should work
    #[test]
    fn can_commit_link() {
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex");
        let link = create_test_link();
        let link_list_entry = LinkListEntry::new(&[link]);
        let (entry_type, entry) = link_list_entry.to_entry();
        let commit_action = ActionWrapper::new(Action::Commit(entry_type, entry));
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
                Action::Commit(entry_type, entry) => {
                    assert_eq!(entry_type, &EntryType::LinkList,);
                    assert_eq!(entry.content(), link_list_entry.to_entry().1.content());
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
        let link_a = create_test_link_a();
        let link_b = create_test_link_b();
        let link_c = create_test_link_c();
        let link_list_entry = LinkListEntry::new(&[link_a, link_b, link_c]);
        let (entry_type, entry) = link_list_entry.to_entry();
        let commit_action = ActionWrapper::new(Action::Commit(entry_type, entry));
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
                Action::Commit(entry_type, entry) => {
                    assert_eq!(entry_type, &EntryType::LinkList,);
                    assert_eq!(entry.content(), link_list_entry.to_entry().1.content());
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
        let lle_entry = lle.to_entry().1;
        let lle_trip = LinkListEntry::from_entry(&lle_entry);
        assert_eq!(lle, lle_trip);
    }
}
