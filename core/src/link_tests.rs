//-------------------------------------------------------------------------------------------------
// Tests
//-------------------------------------------------------------------------------------------------
#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    use action::{Action, ActionWrapper};
    use holochain_core_types::{
        cas::content::{Address, AddressableContent},
        entry::ToEntry,
        entry_type::EntryType,
        links_entry::*,
    };
    use instance::{tests::test_context, Instance, Observer};

    use std::sync::mpsc::channel;

    pub fn create_test_link() -> Link {
        Link::new(
            &Address::from("12".to_string()),
            &Address::from("34".to_string()),
            "fake",
        )
    }

    pub fn create_test_link_a() -> Link {
        create_test_link()
    }

    pub fn create_test_link_b() -> Link {
        Link::new(
            &Address::from("56".to_string()),
            &Address::from("78".to_string()),
            "faux",
        )
    }

    pub fn create_test_link_c() -> Link {
        Link::new(
            &Address::from("90".to_string()),
            &Address::from("ab".to_string()),
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
        let entry = link_list_entry.to_entry();
        let commit_action = ActionWrapper::new(Action::Commit(entry));
        // Set up instance and process the action
        let instance = Instance::new(test_context("jason"));
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
                    assert_eq!(entry.entry_type(), &EntryType::LinkList,);
                    assert_eq!(entry.content(), link_list_entry.to_entry().content());
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
        let entry = link_list_entry.to_entry();
        let commit_action = ActionWrapper::new(Action::Commit(entry));
        println!("commit_multilink: {:?}", commit_action);
        // Set up instance and process the action
        let instance = Instance::new(test_context("jason"));
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
                    assert_eq!(entry.entry_type(), &EntryType::LinkList,);
                    assert_eq!(entry.content(), link_list_entry.to_entry().content());
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
