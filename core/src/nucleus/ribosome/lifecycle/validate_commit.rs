use super::call;
use action::ActionWrapper;
use instance::Observer;
use nucleus::ribosome::lifecycle::{
    LifecycleFunction, LifecycleFunctionParams, LifecycleFunctionResult,
};
use std::sync::mpsc::Sender;

pub fn validate_commit(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: &str,
    params: &LifecycleFunctionParams,
) -> LifecycleFunctionResult {
    call(
        action_channel,
        observer_channel,
        zome,
        &LifecycleFunction::ValidateCommit,
        params,
    )
}

#[cfg(test)]
pub mod tests {

    use super::validate_commit;
    use hash_table::entry::tests::test_entry;
    use holochain_wasm_utils::HcApiReturnCode;
    use nucleus::ribosome::{
        lifecycle::{
            tests::test_lifecycle_function_instance, LifecycleFunction, LifecycleFunctionParams,
            LifecycleFunctionResult,
        },
        Defn,
    };

    #[test]
    fn pass() {
        let zome = "test_zome";
        let instance =
            test_lifecycle_function_instance(zome, LifecycleFunction::ValidateCommit.as_str(), 0);

        let result = validate_commit(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &LifecycleFunctionParams::ValidateCommit(test_entry()),
        );

        assert_eq!(LifecycleFunctionResult::Pass, result);
    }

    #[test]
    fn not_implemented() {
        let zome = "test_zome";
        let instance = test_lifecycle_function_instance(
            zome,
            // anything other than ValidateCommit is fine here
            LifecycleFunction::Genesis.as_str(),
            0,
        );

        let result = validate_commit(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &LifecycleFunctionParams::ValidateCommit(test_entry()),
        );

        assert_eq!(LifecycleFunctionResult::NotImplemented, result);
    }

    #[test]
    fn fail() {
        let zome = "test_zome";
        let instance =
            test_lifecycle_function_instance(zome, LifecycleFunction::ValidateCommit.as_str(), 1);

        let result = validate_commit(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &LifecycleFunctionParams::ValidateCommit(test_entry()),
        );

        // @TODO how to get fail strings back out?
        // @see https://github.com/holochain/holochain-rust/issues/205
        assert_eq!(LifecycleFunctionResult::Fail("{".to_string()), result);
    }

}
