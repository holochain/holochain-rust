use hash_table::entry::Entry;
use action::Action;
use action::ActionResult;

/// Action

pub struct Commit {
    entry: Entry,
}

impl Action for Commit {}

impl Commit {

    pub fn new(entry: &Entry) -> Commit {
        Commit{
            entry,
        }
    }

}

/// ActionResult

pub struct CommitResult {
    value: String,
}

impl ActionResult for CommitResult {}

#[cfg(test)]
pub mod tests {

    /// builds a dummy action for testing commit
    pub fn test_action_commit() -> Action {
        Action::commit(&test_entry())
    }

    /// builds a dummy action result for testing commit
    pub fn test_action_result_commit() -> ActionResult {
        ActionResult::Commit(test_entry().key())
    }

    #[test]
    /// smoke test building a new commit action + result
    fn action_commit() {
        test_action_commit();
        test_action_result_commit();

        // actions have unique ids and are not equal
        assert_ne!(test_action_commit(), test_action_commit());
        // the result is equal though
        assert_eq!(test_action_result_commit(), test_action_result_commit());
    }

}
