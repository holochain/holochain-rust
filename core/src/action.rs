use std::hash::Hash;
use std::hash::Hasher;
use hash_table::entry::Entry;
use nucleus::FunctionCall;
use nucleus::FunctionResult;
use nucleus::EntrySubmission;
use snowflake;
use holochain_dna::Dna;

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

    Commit(Entry),
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
        Action{
            signal: signal.clone(),
            id: snowflake::ProcessUniqueId::new(),
        }
    }

    pub fn signal(&self) -> Signal {
        self.signal.clone()
    }

}

impl Eq for Action {}
