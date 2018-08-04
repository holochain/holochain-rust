pub mod commit;
pub mod get;
pub mod execute_zome_function;
pub mod return_initialization_result;
pub mod return_zome_function_result;

use snowflake;

pub trait Action: Eq {

    fn id() -> snowflake::ProcessUniqueId {
        snowflake::ProcessUniqueId::new()
    }

}

pub trait ActionResult: Eq {

}

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
