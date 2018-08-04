use action::Action;
use hash_table::pair::Pair;
use action::ActionResult;

/// Action

pub struct Get {
    key: String,
}

impl Action for Get {}

impl Get {

    pub fn new(key: &str) -> Get {
        Get{
            key: key.clone(),
        }
    }

}

/// ActionResult

pub struct GetResult {
    value: Option<Pair>
}

impl ActionResult for GetResult {}

#[cfg(test)]
pub mod tests {

    /// builds a dummy action for testing get
    pub fn test_action_get() -> Action {
        Action::get(&test_entry().key())
    }

    /// builds a dummy action result for testing get
    pub fn test_action_result_get() -> ActionResult {
        ActionResult::Get(Some(test_pair()))
    }

    #[test]
    /// smoke test building a new get action + result
    fn action_get() {
        test_action_get();
        test_action_result_get();

        // actions have unique ids and are not equal
        assert_ne!(test_action_get(), test_action_get());
        // the result is equal though
        assert_eq!(test_action_result_get(), test_action_result_get());
    }

}
