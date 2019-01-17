use super::call;
use crate::{
    context::Context,
    nucleus::ribosome::callback::{Callback, CallbackParams, CallbackResult},
};
use std::sync::Arc;

pub fn genesis(
    context: Arc<Context>,
    zome: &str,
    // we ignore params for genesis
    params: &CallbackParams,
) -> CallbackResult {
    call(context, zome, &Callback::Genesis, params)
}

#[cfg(test)]
pub mod tests {

    use super::genesis;
    use crate::{
        instance::tests::test_context,
        nucleus::ribosome::{
            callback::{tests::test_callback_instance, Callback, CallbackParams, CallbackResult},
            Defn,
        },
    };

    #[test]
    fn pass() {
        let zome = "test_zome";
        let instance = test_callback_instance(zome, Callback::Genesis.as_str(), 0)
            .expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test"));

        let result = genesis(context, zome, &CallbackParams::Genesis);

        assert_eq!(CallbackResult::Pass, result);
    }

    #[test]
    fn not_implemented() {
        let zome = "test_zome";
        let instance = test_callback_instance(
            zome,
            // anything other than Genesis is fine here
            Callback::Receive.as_str(),
            0,
        )
        .expect("Test callback instance could not be initialized");

        let context = instance.initialize_context(test_context("test"));

        let result = genesis(context, zome, &CallbackParams::Genesis);

        if let CallbackResult::NotImplemented(_) = result {
            ()
        } else {
            panic!("unexpected result");
        }
    }

    #[test]
    fn fail() {
        let zome = "test_zome";
        let instance = test_callback_instance(zome, Callback::Genesis.as_str(), 1);
        assert!(instance.is_err());
        let error = instance.err().unwrap();
        assert_eq!("\"".to_string(), error);
    }

}
