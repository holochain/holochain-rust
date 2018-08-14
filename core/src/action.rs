use agent::state::AgentState;
use context::Context;
use hash_table::entry::Entry;
use holochain_dna::Dna;
use instance::Observer;
use nucleus::{state::NucleusState, EntrySubmission, FunctionCall, FunctionResult};
use snowflake;
use std::{
    hash::{Hash, Hasher},
    sync::{mpsc::Sender, Arc},
};

#[derive(Clone, Debug)]
// @TODO what is wrapper for?
// https://github.com/holochain/holochain-rust/issues/192
pub struct ActionWrapper {
    action: Action,
    id: snowflake::ProcessUniqueId,
}

impl ActionWrapper {
    /// constructor from &Action
    /// internal snowflake ID is automatically set
    pub fn new(a: &Action) -> Self {
        ActionWrapper {
            action: a.clone(),
            // auto generate id
            id: snowflake::ProcessUniqueId::new(),
        }
    }

    /// read only access to action
    pub fn action(&self) -> Action {
        self.action.clone()
    }

    /// read only access to id
    pub fn id(&self) -> snowflake::ProcessUniqueId {
        self.id
    }
}

impl PartialEq for ActionWrapper {
    fn eq(&self, other: &ActionWrapper) -> bool {
        self.id == other.id
    }
}

impl Eq for ActionWrapper {}

impl Hash for ActionWrapper {
    /// @TODO dangerous when persisted!
    /// snowflake only guarantees uniqueness per process
    /// @see https://github.com/holochain/holochain-rust/issues/203
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Clone, PartialEq, Hash, Debug)]
pub enum Action {
    /// entry to Commit
    /// MUST already have passed all callback checks
    Commit(Entry),
    /// hash to Get
    Get(String),

    /// execute a function in a zome WASM
    ExecuteZomeFunction(FunctionCall),
    /// return the result of a zome WASM function call
    ReturnZomeFunctionResult(FunctionResult),

    /// initialize an application from a Dna
    /// not the same as genesis
    /// may call genesis internally
    InitApplication(Dna),
    /// return the result of an InitApplication action
    ReturnInitializationResult(Option<String>),

    /// ???
    // @TODO how does this relate to validating a commit?
    ValidateEntry(EntrySubmission),

    /// add a network peer
    AddPeer(String),
}

/// function signature for action handler functions
// @TODO merge these into a single signature
// @see https://github.com/holochain/holochain-rust/issues/194
pub type AgentReduceFn =
    fn(&mut AgentState, &ActionWrapper, &Sender<ActionWrapper>, &Sender<Observer>);
pub type NucleusReduceFn =
    fn(Arc<Context>, &mut NucleusState, &ActionWrapper, &Sender<ActionWrapper>, &Sender<Observer>);

#[cfg(test)]
pub mod tests {

    use action::{Action, ActionWrapper};
    use hash::tests::test_hash;
    use hash_table::entry::tests::{test_entry, test_entry_hash};
    use nucleus::tests::test_function_result;

    /// dummy action
    pub fn test_action() -> Action {
        Action::Get(test_entry_hash())
    }

    /// dummy action wrapper with test_action()
    pub fn test_action_wrapper() -> ActionWrapper {
        ActionWrapper::new(&test_action())
    }

    /// dummy action wrapper with commit of test_entry()
    pub fn test_action_wrapper_commit() -> ActionWrapper {
        ActionWrapper::new(&Action::Commit(test_entry()))
    }

    /// dummy action for a get of test_hash()
    pub fn test_action_wrapper_get() -> ActionWrapper {
        ActionWrapper::new(&Action::Get(test_hash()))
    }

    pub fn test_action_wrapper_rzfr() -> ActionWrapper {
        ActionWrapper::new(&Action::ReturnZomeFunctionResult(test_function_result()))
    }

    #[test]
    /// smoke test actions
    pub fn new_action() {
        let a1 = test_action();
        let a2 = test_action();

        // unlike actions and wrappers, signals are equal to themselves
        assert_eq!(a1, a2);
    }

    #[test]
    /// tests that new action wrappers take an action and ensure uniqueness
    pub fn new_action_wrapper() {
        let aw1 = test_action_wrapper();
        let aw2 = test_action_wrapper();

        // snowflake enforces uniqueness
        assert_eq!(aw1, aw1);
        assert_ne!(aw1, aw2);
    }

    #[test]
    /// tests read access to actions
    pub fn action_signal() {
        let aw1 = test_action_wrapper();
        let aw2 = test_action_wrapper();

        assert_eq!(aw1.action(), aw2.action());
        assert_eq!(aw1.action(), test_action());
    }

}
