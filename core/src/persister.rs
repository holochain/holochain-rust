use error::HolochainError;
use state::State;

/// trait that defines the persistence functionality that holochain_core requires
pub trait Persister: Send {
    // @TODO how does save/load work with snowflake IDs?
    // snowflake is only unique across a single process, not a reboot save/load round trip
    // we'd need real UUIDs for persistant uniqueness
    // @see https://github.com/holochain/holochain-rust/issues/203
    fn save(&mut self, state: State);
    fn load(&self) -> Result<Option<State>, HolochainError>;
}

#[derive(Default, Clone, PartialEq)]
pub struct SimplePersister {
    state: Option<State>,
}

impl Persister for SimplePersister {
    fn save(&mut self, state: State) {
        self.state = Some(state);
    }
    fn load(&self) -> Result<Option<State>, HolochainError> {
        Ok(self.state.clone())
    }
}

impl SimplePersister {
    pub fn new() -> Self {
        SimplePersister { state: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use action::{tests::test_action_wrapper_commit, ActionWrapper};
    use instance::tests::test_context;
    use std::sync::mpsc::channel;

    #[test]
    fn can_instantiate() {
        let store = SimplePersister::new();

        assert_eq!(store.load(), Ok(None));
    }

    #[test]
    fn can_roundtrip() {
        let mut store = SimplePersister::new();

        let state = State::new();

        let action_wrapper = test_action_wrapper_commit();

        let (sender, _receiver) = channel::<ActionWrapper>();
        let (tx_observer, _observer) = channel::<::instance::Observer>();
        let new_state = state.reduce(
            test_context("jane"),
            action_wrapper.clone(),
            &sender,
            &tx_observer,
        );

        store.save(new_state.clone());

        assert_eq!(store.load(), Ok(Some(new_state)));
    }
}
