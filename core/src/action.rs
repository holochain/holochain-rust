use hash_table::entry::Entry;
use holochain_dna::Dna;
use nucleus::{EntrySubmission, FunctionCall, FunctionResult};
use snowflake;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
// @TODO what is wrapper for?
// https://github.com/holochain/holochain-rust/issues/192
pub struct ActionWrapper {
    pub action: Action,
    pub id: snowflake::ProcessUniqueId,
}

impl ActionWrapper {
    pub fn new(a: Action) -> Self {
        ActionWrapper {
            action: a,
            id: snowflake::ProcessUniqueId::new(),
        }
    }
}

impl PartialEq for ActionWrapper {
    fn eq(&self, other: &ActionWrapper) -> bool {
        self.id == other.id
    }
}

impl Eq for ActionWrapper {}

impl Hash for ActionWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(PartialEq, Clone, Hash, Debug)]
pub struct Action {
    signal: Signal,
    id: snowflake::ProcessUniqueId,
}

impl Action {
    pub fn new(signal: &Signal) -> Action {
        Action {
            signal: signal.clone(),
            id: snowflake::ProcessUniqueId::new(),
        }
    }

    pub fn signal(&self) -> Signal {
        self.signal.clone()
    }
}

impl Eq for Action {}

#[derive(Clone, PartialEq, Hash, Debug)]
pub enum Signal {
    /// entry to commit
    /// MUST already have passed all lifecycle checks
    Commit(Entry),
    Get(String),

    ExecuteZomeFunction(FunctionCall),
    InitApplication(Dna),
    ReturnInitializationResult(Option<String>),
    ReturnZomeFunctionResult(FunctionResult),
    ValidateEntry(EntrySubmission),

    AddPeer(String),
}

#[cfg(test)]
pub mod tests {

    use action::{Action, Signal};
    use hash_table::entry::tests::test_entry;
    use hash_table::entry::tests::test_entry_hash;

    pub fn test_action_commit() -> Action {
        Action::new(&Signal::Commit(test_entry()))
    }

    pub fn test_signal() -> Signal {
        Signal::Get(test_entry_hash())
    }

    pub fn test_action() -> Action {
        Action::new(&test_signal())
    }

    #[test]
    /// tests that new actions take a signal and ensure uniqueness
    pub fn action_new() {
        let a1 = test_action();
        let a2 = test_action();

        // snowflake enforces uniqueness
        assert_eq!(a1, a1);
        assert_ne!(a1, a2);
    }

    #[test]
    /// tests read access to action signals
    pub fn action_signal() {
        let a1 = test_action();
        let a2 = test_action();

        assert_eq!(a1.signal(), a2.signal());
        assert_eq!(a1.signal(), test_signal());
    }

}
