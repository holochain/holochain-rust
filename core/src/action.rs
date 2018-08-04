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

// #[derive(Clone, Debug, PartialEq)]
// pub enum ActionResult {
//
//     Commit(commit::CommitResult),
//     Get(get::GetResult),
//
// }

#[cfg(test)]
pub mod tests {

    /// provides a dummy action for testing not associated with a real action
    pub struct TestAction {
        value: i32,
    }

    /// use the default Action implementation
    impl Action for TestAction {}

    impl TestAction {

        /// given an i32, returns a TestAction
        pub fn new(i: i32) {
            TestAction {
                value: i,
            }
        }

    }

    /// dummy TestAction
    pub fn test_test_action() {
        TestAction::new(42)
    }

    #[test]
    /// tests the default id implementation
    fn id() {
        assert_ne!(test_test_action().id(), test_test_action().id());
    }

}
