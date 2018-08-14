use super::call;
use action::ActionWrapper;
use instance::Observer;
use nucleus::ribosome::callback::{Callback, CallbackParams, CallbackResult};
use std::sync::mpsc::Sender;

pub fn genesis(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: &str,
    // we ignore params for genesis
    params: &CallbackParams,
) -> CallbackResult {
    call(
        action_channel,
        observer_channel,
        zome,
        &Callback::Genesis,
        params,
    )
}

#[cfg(test)]
pub mod tests {

    use super::genesis;
    use nucleus::ribosome::{
        callback::{tests::test_callback_instance, Callback, CallbackParams, CallbackResult},
        Defn,
    };

    #[test]
    fn pass() {
        let zome = "test_zome";
        let instance = test_callback_instance(zome, Callback::Genesis.as_str(), 0);

        let result = genesis(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &CallbackParams::Genesis,
        );

        assert_eq!(CallbackResult::Pass, result);
    }

    #[test]
    fn not_implemented() {
        let zome = "test_zome";
        let instance = test_callback_instance(
            zome,
            // anything other than Genesis is fine here
            Callback::ValidateCommit.as_str(),
            0,
        );

        let result = genesis(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &CallbackParams::Genesis,
        );

        assert_eq!(CallbackResult::NotImplemented, result);
    }

    #[test]
    fn fail() {
        let zome = "test_zome";
        let instance = test_callback_instance(zome, Callback::Genesis.as_str(), 1);

        let result = genesis(
            &instance.action_channel(),
            &instance.observer_channel(),
            zome,
            &CallbackParams::Genesis,
        );

        // @TODO how to get fail strings back out?
        // @see https://github.com/holochain/holochain-rust/issues/205
        assert_eq!(CallbackResult::Fail("\u{0}".to_string()), result);
    }

}
