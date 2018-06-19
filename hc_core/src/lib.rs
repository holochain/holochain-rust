#![cfg_attr(feature = "strict", deny(warnings))]
#![feature(fnbox)]

pub mod agent;
pub mod source_chain;
pub mod common;
pub mod instance;
pub mod network;
pub mod nucleus;
pub mod state;

#[cfg(test)]
mod tests {
    //use agent::Action::*;
    use instance::Instance;
    use nucleus::dna::*;
    use nucleus::Action::*;
    use state::Action::*;
    use state::State;
    use std::sync::mpsc::channel;

    #[test]
    fn adding_messages_to_queue() {
        let mut instance = Instance::create();
        instance.start_action_loop();

        let dna = DNA {};
        let (sender, receiver) = channel();
        instance.dispatch_with_observer(Nucleus(InitApplication(dna.clone())), move |state: &State| {
            match state.nucleus().dna() {
                Some(dna) => {
                    sender.send(dna).expect("test channel must be open");
                    return true;
                },
                None => return false
            }
        });

        let stored_dna = receiver.recv().unwrap();

        assert_eq!(
            dna,
            stored_dna
        );
/*
        let entry = ::common::entry::Entry::new(&String::new());
        let action = Agent(Commit(entry));
        instance.dispatch(action.clone(), None);
        assert_eq!(*instance.pending_actions().back().unwrap(), action);*/
    }
/*
    #[test]
    fn consuming_actions_and_checking_state_mutation() {
        let mut instance = Instance::create();
        assert_eq!(instance.state().nucleus().dna(), None);
        assert_eq!(instance.state().nucleus().inits(), 0);

        let dna = DNA {};
        let action = Nucleus(InitApplication(dna.clone()));
        instance.dispatch(action.clone(), None);
        instance.consume_next_action();

        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert_eq!(instance.state().nucleus().inits(), 1);

        instance.dispatch(action.clone(), None);
        instance.consume_next_action();

        assert_eq!(instance.state().nucleus().inits(), 2);
    }*/
}
