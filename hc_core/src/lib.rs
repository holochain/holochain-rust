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

    // This test shows how to call dispatch with a closure that should run
    // when the action results in a state change.  Note that the observer closure
    // needs to return a boolean to indicate that it has successfully observed what
    // it intends to observe.  It will keep getting called as the state changes until
    // it returns true.
    // Note also that for this test we create a channel to send something (in this case
    // the dna) back over, just so that the test will block until the closure is successfully
    // run and the assert will actually run.  If we put the assert inside the closure
    // the test thread could complete before the closure was called.
    #[test]
    fn dispatch_with_observer() {
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
    }

    #[test]
    fn dispatch_and_wait() {
        let mut instance = Instance::new();
        assert_eq!(instance.state().nucleus().dna(), None);
        assert_eq!(instance.state().nucleus().initialized(), false);

        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop();
        instance.dispatch_and_wait(action.clone());

        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert_eq!(instance.state().nucleus().initialized(), true);

        instance.dispatch_and_wait(action.clone());
        assert_eq!(instance.state().nucleus().initialized(), true);
    }
}
