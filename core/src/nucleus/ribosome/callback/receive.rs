use super::call;
use context::Context;
use holochain_core_types::ribosome::callback::{CallbackParams, CallbackResult};
use nucleus::ribosome::callback::Callback;
use std::sync::Arc;

pub fn receive(
    context: Arc<Context>,
    zome: &str,
    // we ignore params for genesis
    params: &CallbackParams,
) -> CallbackResult {
    call(context, zome, &Callback::Receive, params)
}

#[cfg(test)]
pub mod tests {

    use super::receive;
    use instance::tests::test_context;
    use nucleus::ribosome::{
        callback::{tests::test_callback_instance, Callback, CallbackParams, CallbackResult},
        Defn,
    };

    #[test]
    fn not_implemented() {
        let zome = "test_zome";
        let instance = test_callback_instance(
            zome,
            // anything other than Genesis is fine here
            Callback::MissingNo.as_str(),
            0,
        ).expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test"));

        let result = receive(context, zome, &CallbackParams::Receive);

        assert_eq!(CallbackResult::NotImplemented, result);
    }

    #[test]
    fn pass_test() {
        let zome = "test_zome";
        let instance = test_callback_instance(zome, Callback::Receive.as_str(), 0)
            .expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test"));

        let result = receive(context, zome, &CallbackParams::Receive);

        assert_eq!(CallbackResult::Pass, result);
    }

    #[test]
    fn fail_test() {
        let zome = "test_zome";
        let instance = test_callback_instance(zome, Callback::Receive.as_str(), 1)
            .expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test"));

        let result = receive(context, zome, &CallbackParams::Receive);

        // @TODO how to get fail strings back out?
        // @see https://github.com/holochain/holochain-rust/issues/205
        assert_eq!(CallbackResult::Fail("\"".to_string()), result);
    }

}
