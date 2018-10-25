use holochain_core_types::error::HolochainError;
use state::State;
use std::fs::OpenOptions;

/// trait that defines the persistence functionality that holochain_core requires
pub trait Persister: Send {
    // @TODO how does save/load work with snowflake IDs?
    // snowflake is only unique across a single process, not a reboot save/load round trip
    // we'd need real UUIDs for persistant uniqueness
    // @see https://github.com/holochain/holochain-rust/issues/203
    fn save(&mut self, state: State)->Result<(),HolochainError>;
    fn load(&self,context:Arc<Context>) -> Result<Option<State>, HolochainError>;
}

#[derive(Default, Clone, PartialEq)]
pub struct SimplePersister {
    state: Option<State>,
    file_path : String
}

impl Persister for SimplePersister {
    fn save(&mut self, state: State)->Result<(),HolochainError> {
        let mut f = OpenOptions::new().write(true).create(file_path).open(self.file_path);
        let json = State::deserialize_state(state)?;
        Ok(f.write_all(json.as_bytes())?)
    }
    fn load(&self,context:Arc<Context>) -> Result<Option<State>, HolochainError> {
        let mut f = File::open(filename)?;
        let mut json = String::new();
        f.read_to_string(&mut json)?;
        let agent = AgentStateSnapshot::deserialize_state(context,json)?;
        Ok(State::new_with_agent(arc,agent))
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

        assert_eq!(store.load(), Ok(None));
    }

    // TODO write a persister.save() test
}
