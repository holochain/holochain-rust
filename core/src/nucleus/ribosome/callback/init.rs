use crate::nucleus::ribosome::callback::run_callback;
// use crate::nucleus::ribosome::callback::call;
use crate::{
    context::Context,
    nucleus::ribosome::callback::{Callback, CallbackFnCall, CallbackParams, CallbackResult, Defn},
};
use std::sync::Arc;

pub fn init(context: Arc<Context>, zome: &str, parameters: &CallbackParams) -> CallbackResult {
    // call(context, zome, &Callback::Init, parameters)
    let params = match parameters {
        CallbackParams::Init(params) => params,
        _ => return CallbackResult::NotImplemented("init/0".into()),
    };
    let call = CallbackFnCall::new(zome, &Callback::Init.as_str().to_string(), params);
    let dna = context.get_dna().expect("Callback called without DNA set!");
    match dna.get_wasm_from_zome_name(zome) {
        None => CallbackResult::NotImplemented("init/1".into()),
        Some(wasm) => {
            if wasm.code.is_empty() {
                CallbackResult::NotImplemented("init/2".into())
            } else {
                run_callback(context.clone(), call)
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::init;
    use crate::{
        instance::tests::test_context,
        nucleus::ribosome::{
            callback::{tests::test_callback_instance, Callback, CallbackParams, CallbackResult},
            Defn,
        },
    };
    use holochain_wasm_utils::api_serialization::init::InitParams;

    #[test]
    fn pass() {
        let zome = "test_zome";
        let netname = Some("init::pass");
        let instance = test_callback_instance(zome, Callback::Init.as_str(), 0, netname)
            .expect("Test callback instance could not be initialized");
        let context = instance.initialize_context(test_context("test", netname));

        let result = init(context, zome, &CallbackParams::Init(InitParams::default()));

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

        let result = init(context, zome, &CallbackParams::Init(InitParams::default()));

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
            "At least one zome init returned error: [(\"test_zome\", \"{\")]".to_string(),
            error
        );
    }

}
