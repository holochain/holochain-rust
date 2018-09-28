use super::call;
use context::Context;
use nucleus::ribosome::callback::{Callback, CallbackParams, CallbackResult};
use std::sync::Arc;

pub fn validate_commit(
    context: Arc<Context>,
    zome: &str,
    params: &CallbackParams,
) -> CallbackResult {
    call(context, zome, &Callback::ValidateCommit, params)
}

#[cfg(test)]
pub mod tests {

    use super::validate_commit;
    use hash_table::entry::tests::test_entry;
    use instance::tests::test_context;
    use nucleus::ribosome::{
        callback::{tests::test_callback_instance, Callback, CallbackParams, CallbackResult},
        Defn,
    };

    #[test]
    fn pass() {
        let zome = "test_zome";
        let instance = test_callback_instance(zome, Callback::ValidateCommit.as_str(), 0);
        let context = instance.initialize_context(test_context("test"));

        let result = validate_commit(context, zome, &CallbackParams::ValidateCommit(test_entry()));

        assert_eq!(CallbackResult::Pass, result);
    }

    #[test]
    fn not_implemented() {
        let zome = "test_zome";
        let instance = test_callback_instance(
            zome,
            // anything other than ValidateCommit is fine here
            Callback::Genesis.as_str(),
            0,
        );
        let context = instance.initialize_context(test_context("test"));

        let result = validate_commit(context, zome, &CallbackParams::ValidateCommit(test_entry()));

        assert_eq!(CallbackResult::NotImplemented, result);
    }

    #[test]
    fn fail() {
        let zome = "test_zome";
        let instance = test_callback_instance(zome, Callback::ValidateCommit.as_str(), 1);
        let context = instance.initialize_context(test_context("test"));

        let result = validate_commit(context, zome, &CallbackParams::ValidateCommit(test_entry()));

        // @TODO how to get fail strings back out?
        // @see https://github.com/holochain/holochain-rust/issues/205
        assert_eq!(CallbackResult::Fail("{".to_string()), result);
    }

}
