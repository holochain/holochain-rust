//-------------------------------------------------------------------------------------------------
// Tests
//-------------------------------------------------------------------------------------------------
#[cfg(test)]
pub mod tests {
    use crate::{
        action::{Action, ActionWrapper},
        instance::{tests::test_context, Instance},
    };
    use holochain_core_types::{
        entry::Entry,
        link::{link_list::LinkList, Link},
    };
    use holochain_persistence_api::cas::content::{Address, AddressableContent};

    pub fn create_example_link() -> Link {
        Link::new(
            &Address::from("12".to_string()),
            &Address::from("34".to_string()),
            "fake",
            "fake-tag",
        )
    }

    pub fn create_test_link_a() -> Link {
        create_example_link()
    }

    pub fn create_test_link_b() -> Link {
        Link::new(
            &Address::from("56".to_string()),
            &Address::from("78".to_string()),
            "faux",
            "fake-tag",
        )
    }

    pub fn create_test_link_c() -> Link {
        Link::new(
            &Address::from("90".to_string()),
            &Address::from("ab".to_string()),
            "fake",
            "fake-tag",
        )
    }

    /// Committing a LinkEntry to source chain should work
    #[test]
    fn can_commit_link() {
        let netname = Some("can_commit_link");
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex", netname);
        let link = create_example_link();
        let link_list = LinkList::new(&[link]);
        let link_list_entry = Entry::LinkList(link_list);
        let commit_action =
            ActionWrapper::new(Action::Commit((link_list_entry.clone(), None, vec![])));
        // Set up instance and process the action
        let instance = Instance::new(test_context("jason", netname));
        let context = instance.context();
        instance
            .process_action(&commit_action, &context)
            .expect("process_action should run without error");
        // Check if LinkEntry is found
        assert!(instance
            .state()
            .agent()
            .iter_chain()
            .any(|header| *header.entry_address() == link_list_entry.address()))
    }

    /// Committing a LinkList to source chain should work
    #[test]
    fn can_commit_multilink() {
        let netname = Some("can_commit_multilink");
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex", netname);
        let link_a = create_test_link_a();
        let link_b = create_test_link_b();
        let link_c = create_test_link_c();
        let link_list = LinkList::new(&[link_a, link_b, link_c]);
        let link_list_entry = Entry::LinkList(link_list.clone());
        let commit_action =
            ActionWrapper::new(Action::Commit((link_list_entry.clone(), None, vec![])));
        println!("commit_multilink: {:?}", commit_action);
        // Set up instance and process the action
        let instance = Instance::new(test_context("jason", netname));
        let context = instance.context();
        instance
            .process_action(&commit_action, &context)
            .expect("process_action should run without error");
        // Check if LinkEntry is found
        assert!(instance
            .state()
            .agent()
            .iter_chain()
            .any(|header| *header.entry_address() == link_list_entry.address()))
    }
}
