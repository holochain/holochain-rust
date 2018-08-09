use super::call;
use action::ActionWrapper;
use instance::Observer;
use nucleus::ribosome::lifecycle::{
    LifecycleFunction, LifecycleFunctionParams, LifecycleFunctionResult,
};
use std::sync::mpsc::Sender;

pub fn genesis(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: &str,
    // we ignore params for genesis
    params: &LifecycleFunctionParams,
) -> LifecycleFunctionResult {
    call(
        action_channel,
        observer_channel,
        zome,
        &LifecycleFunction::Genesis,
        params,
    )
}

#[cfg(test)]
pub mod tests {

    use super::genesis;
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
            test_lifecycle_function_instance(zome, LifecycleFunction::Genesis.as_str(), 0);

        let result = genesis(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &LifecycleFunctionParams::Genesis,
        );

        assert_eq!(LifecycleFunctionResult::Pass, result);
    }

    #[test]
    fn not_implemented() {
        let zome = "test_zome";
        let instance = test_lifecycle_function_instance(
            zome,
            // anything other than Genesis is fine here
            LifecycleFunction::ValidateCommit.as_str(),
            0,
        );

        let result = genesis(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &LifecycleFunctionParams::Genesis,
        );

        assert_eq!(LifecycleFunctionResult::NotImplemented, result);
    }

    #[test]
    fn fail() {
        let zome = "test_zome";
        let instance =
            test_lifecycle_function_instance(zome, LifecycleFunction::Genesis.as_str(), 1);

        let result = genesis(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &LifecycleFunctionParams::Genesis,
        );

        // @TODO how to get fail strings back out?
        // @see https://github.com/holochain/holochain-rust/issues/205
        assert_eq!(LifecycleFunctionResult::Fail("\u{0}".to_string()), result);
    }

}
