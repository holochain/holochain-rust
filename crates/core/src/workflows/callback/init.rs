use crate::{
    context::Context,
    wasm_engine::callback::{call, Callback, CallbackParams, CallbackResult},
    NEW_RELIC_LICENSE_KEY,
};
use std::sync::Arc;

#[autotrace]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn init(
    context: Arc<Context>,
    zome: &str,
    // we ignore params for init
    params: &CallbackParams,
) -> CallbackResult {
    call(context, zome, &Callback::Init, params)
}

#[cfg(test)]
pub mod tests {

    use super::init;
    use crate::{
        instance::tests::test_context,
        wasm_engine::{
            callback::{tests::test_callback_instance, Callback, CallbackParams, CallbackResult},
            Defn,
        },
    };

    #[test]
    fn pass() {
        let zome = "test_zome";
        let netname = Some("init::pass");
        let instance = test_callback_instance(zome, Callback::Init.as_str(), 0, netname)
            .expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test", netname));

        let result = init(context, zome, &CallbackParams::Init);

        assert_eq!(CallbackResult::Pass, result);
    }

    #[test]
    fn not_implemented() {
        let zome = "test_zome";
        let netname = Some("init::not_implemented");
        let instance = test_callback_instance(
            zome,
            // anything other than init is fine here
            Callback::Receive.as_str(),
            0,
            netname,
        )
        .expect("Test callback instance could not be initialized");

        let context = instance.initialize_context(test_context("test", netname));

        let result = init(context, zome, &CallbackParams::Init);

        if let CallbackResult::NotImplemented(_) = result {
            ()
        } else {
            panic!("unexpected result");
        }
    }

    #[test]
    fn fail() {
        let zome = "test_zome";
        let netname = Some("init::fail");
        let instance = test_callback_instance(zome, Callback::Init.as_str(), 1, netname);
        assert!(instance.is_err());
        let error = instance.err().unwrap();
        assert_eq!(
            "At least one zome init returned error: [(\"test_zome\", \"\\\"\")]".to_string(),
            error
        );
    }
}
