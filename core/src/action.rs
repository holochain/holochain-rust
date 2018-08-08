use hash_table::entry::Entry;
use holochain_dna::Dna;
use nucleus::{EntrySubmission, FunctionCall, FunctionResult};
use snowflake;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
// @TODO what is wrapper for?
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

#[derive(Clone, PartialEq, Hash, Debug)]
pub enum Signal {
    /// function runtime that triggered the commit signal
    /// needed to chain results, e.g. validate_commit
    /// candidate entry to committed
    /// failed validation will prevent the commit
    Commit(FunctionCall, Entry),
    Get(String),

    ExecuteZomeFunction(FunctionCall),
    InitApplication(Dna),
    ReturnInitializationResult(Option<String>),
    ReturnZomeFunctionResult(FunctionResult),
    ValidateEntry(EntrySubmission),
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

#[cfg(test)]
pub mod tests {

    use action::{Action, Signal};
    use hash_table::entry::tests::test_entry;
    use nucleus::FunctionCall;

    pub fn test_action_commit() -> Action {
        let fc = FunctionCall::new("commit test zome", "", "some_function_calling_commit", "");
        Action::new(&Signal::Commit(fc, test_entry()))
    }

}
