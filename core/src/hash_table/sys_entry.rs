//-------------------------------------------------------------------------------------------------
// UNIT TESTS
//-------------------------------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
    extern crate test_utils;

    use action::{Action, ActionWrapper};
    use holochain_core_types::{
        cas::content::AddressableContent,
        to_entry::ToEntry,
        entry_type::EntryType,
    };

    use instance::{tests::test_context, Instance, Observer};
    use std::sync::mpsc::channel;

    /// Committing a DnaEntry to source chain should work
    #[test]
    fn can_commit_dna() {
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex");
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let (dna_entry_type, dna_entry) = dna.to_entry();
        let commit_action = ActionWrapper::new(Action::Commit(dna_entry_type, dna_entry.clone()));

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
                Action::Commit(entry_type, entry) => {
                    assert_eq!(entry_type, &EntryType::Dna);
                    assert_eq!(entry.content(), dna_entry.content());
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
        let (agent_entry_type, agent_entry) = context.agent.to_entry();
        let commit_agent_action =
            ActionWrapper::new(Action::Commit(agent_entry_type, agent_entry.clone()));

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
                Action::Commit(entry_type, entry) => {
                    assert_eq!(entry_type, &EntryType::AgentId,);
                    assert_eq!(entry.content(), agent_entry.content());
                    true
                }
                _ => false,
            });
    }
}
