#![cfg_attr(feature = "strict", deny(warnings))]
extern crate hc_dna;
pub mod agent;
pub mod common;
pub mod context;
pub mod error;
pub mod instance;
pub mod logger;
pub mod network;
pub mod nucleus;
pub mod persister;
pub mod state;

#[cfg(test)]
mod tests {
    use agent::Action::*;
    use error::*;
    use hc_dna::Dna;
    use instance::Instance;
    use nucleus::Action::*;
    use state::Action::*;

    #[test]
    fn adding_messages_to_queue() {
        let mut instance = Instance::new();

        let dna = Dna::new();
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
        assert_eq!(instance.state().nucleus().initialized(), false);

        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.dispatch(action.clone());

        match instance.consume_next_action() {
            Ok(()) => assert!(true),
            Err(_) => assert!(false),
        };

        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert_eq!(instance.state().nucleus().initialized(), true);

        instance.dispatch(action.clone());
        match instance.consume_next_action() {
            Ok(()) => assert!(true),
            Err(err) => match err {
                HolochainError::AllreadyInitialized => assert!(true),
                _ => assert!(false),
            },
        };

        assert_eq!(instance.state().nucleus().initialized(), true);
    }
}
