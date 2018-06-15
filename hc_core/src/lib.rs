#![cfg_attr(feature = "strict", deny(warnings))]
pub mod agent;
pub mod common;
pub mod instance;
pub mod network;
pub mod nucleus;
pub mod state;
pub mod error;

#[cfg(test)]
mod tests {
    use agent::Action::*;
    use instance::Instance;
    use nucleus::dna::*;
    use nucleus::Action::*;
    use state::Action::*;

    #[test]
    fn adding_messages_to_queue() {
        let mut instance = Instance::new();

        let dna = DNA {};
        instance.dispatch(Nucleus(InitApplication(dna.clone())));
        assert_eq!(
            *instance.pending_actions().back().unwrap(),
            Nucleus(InitApplication(dna.clone()))
        );

        let entry = ::common::entry::Entry {};
        let action = Agent(Commit(entry));
        instance.dispatch(action.clone());
        assert_eq!(*instance.pending_actions().back().unwrap(), action);
    }

    #[test]
    fn consuming_actions_and_checking_state_mutation() {
        let mut instance = Instance::new();
        assert_eq!(instance.state().nucleus().dna(), None);
        assert_eq!(instance.state().nucleus().inits(), 0);

        let dna = DNA {};
        let action = Nucleus(InitApplication(dna.clone()));
        instance.dispatch(action.clone());
        instance.consume_next_action();

        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert_eq!(instance.state().nucleus().inits(), 1);

        instance.dispatch(action.clone());
        instance.consume_next_action();

        assert_eq!(instance.state().nucleus().inits(), 2);
    }
}
