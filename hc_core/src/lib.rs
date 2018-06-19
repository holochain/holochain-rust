#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(clippy)]
pub mod agent;
pub mod source_chain;
pub mod common;
pub mod instance;
pub mod network;
pub mod nucleus;
pub mod state;

#[cfg(test)]
mod tests {
    use agent::Action::*;
    use instance::Instance;
    use nucleus::dna::*;
    use nucleus::Action::*;
    use state::Action::*;

    #[test]
    fn adding_messages_to_queue() {
        let mut instance = Instance::create();

        let dna = DNA {};
        instance.dispatch(Nucleus(InitApplication(dna.clone())));
        assert_eq!(
            *instance.pending_actions().back().unwrap(),
            Nucleus(InitApplication(dna.clone()))
        );

        let entry = ::common::entry::Entry::new(&String::new());
        let action = Agent(Commit(entry));
        instance.dispatch(action.clone());
        assert_eq!(*instance.pending_actions().back().unwrap(), action);
    }

    #[test]
    fn consuming_actions_and_checking_state_mutation() {
        let mut instance = Instance::create();
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
