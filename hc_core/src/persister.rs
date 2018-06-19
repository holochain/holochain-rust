use error::HolochainError;
use state::State;

/// trait that defines the persistence functionality that hc_core requires
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

        let mut state = State::new();

        let entry = ::common::entry::Entry::new(&"some hash".to_string());
        let action = ::state::Action::Agent(::agent::Action::Commit(entry));
        let new_state = state.reduce(&action);

        store.save(&new_state);

        assert_eq!(store.load().unwrap().unwrap(), new_state);
    }
}
