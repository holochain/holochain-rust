use error::HolochainError;
use state::State;

/// trait that defines the persistence functionality that holochain_core requires
pub trait Persister {
    fn save(&mut self, state: &State);
    fn load(&self) -> Result<Option<State>, HolochainError>;
}

#[derive(Default, Clone, PartialEq)]
pub struct SimplePersister {
    state: Option<State>,
}

impl Persister for SimplePersister {
    fn save(&mut self, state: &State) {
        self.state = Some(state.clone());
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
    use std::sync::mpsc::channel;

    #[test]
    fn can_instantiate() {
        let store = SimplePersister::new();
        match store.load() {
            Err(_) => assert!(false),
            Ok(state) => match state {
                None => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn can_roundtrip() {
        let mut store = SimplePersister::new();

        let state = State::new();
        let t = "entryType";

        let entry = ::chain::entry::Entry::new(t, "some hash");
        let action = ::state::Action::Agent(::agent::Action::Commit(entry));
        let (sender, _receiver) = channel::<::state::ActionWrapper>();
        let (tx_observer, _observer) = channel::<::instance::Observer>();
        let new_state = state.reduce(::state::ActionWrapper::new(action), &sender, &tx_observer);

        store.save(&new_state);

        assert_eq!(store.load().unwrap().unwrap(), new_state);
    }
}
