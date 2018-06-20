#![deny(warnings)]
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
pub mod source_chain;
pub mod state;

#[cfg(test)]
mod tests {
    //use agent::Action::*;
    use hc_dna::Dna;
    use instance::Instance;
    use nucleus::Action::*;
    use state::Action::*;
    use state::State;
    use std::sync::mpsc::channel;

    #[test]
    fn adding_messages_to_queue() {
        let mut instance = Instance::new();
        instance.start_action_loop();

        let dna = Dna::new();
        let (sender, receiver) = channel();
        instance.dispatch_with_observer(
            Nucleus(InitApplication(dna.clone())),
            move |state: &State| match state.nucleus().dna() {
                Some(dna) => {
                    sender.send(dna).expect("test channel must be open");
                    return true;
                }
                None => return false,
            },
        );

        let stored_dna = receiver.recv().unwrap();

        assert_eq!(dna, stored_dna);

        /*
        let entry = ::common::entry::Entry::new(&String::new());
        let action = Agent(Commit(entry));
        instance.dispatch(action.clone(), None);
        assert_eq!(*instance.pending_actions().back().unwrap(), action);
        */
    }
    /*
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
            Err(_) => assert!(false),
        };

        assert_eq!(instance.state().nucleus().initialized(), true);

    }*/
}
