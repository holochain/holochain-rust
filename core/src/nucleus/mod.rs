/// Nucleus is the module that handles DNA, including the Ribosome.
///
pub mod actions;
pub mod reducers;
pub mod ribosome;
pub mod state;
pub mod validation;

use holochain_core_types::{
    cas::content::Address, dna::capabilities::CapabilityCall, error::HcResult, json::JsonString,
};
use snowflake;

pub use crate::nucleus::{
    actions::call_zome_function::{call_zome_function, ExecuteZomeFnResponse},
    reducers::reduce,
};

/// Struct holding data for requesting the execution of a Zome function (ExecutionZomeFunction Action)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZomeFnCall {
    id: snowflake::ProcessUniqueId,
    pub zome_name: String,
    pub cap: Option<CapabilityCall>,
    pub fn_name: String,
    pub parameters: JsonString,
}

impl ZomeFnCall {
    pub fn new<J: Into<JsonString>>(
        zome: &str,
        cap: Option<CapabilityCall>,
        function: &str,
        parameters: J,
    ) -> Self {
        ZomeFnCall {
            // @TODO can we defer to the ActionWrapper id?
            // @see https://github.com/holochain/holochain-rust/issues/198
            id: snowflake::ProcessUniqueId::new(),
            zome_name: zome.to_string(),
            cap: cap,
            fn_name: function.to_string(),
            parameters: parameters.into(),
        }
    }

    pub fn same_fn_as(&self, fn_call: &ZomeFnCall) -> bool {
        self.zome_name == fn_call.zome_name
            && self.cap == fn_call.cap
            && self.fn_name == fn_call.fn_name
    }

    pub fn cap_token(&self) -> Address {
        match self.cap.clone() {
            Some(call) => call.cap_token,
            None => panic!("null cap call unimplemented!"),
        }
    }
}

pub type ZomeFnResult = HcResult<JsonString>;

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        instance::{
            tests::{test_context, test_instance_and_context},
            Instance,
        },
        nucleus::{
            call_zome_function,
            state::{NucleusState, NucleusStatus},
        },
    };
    use holochain_core_types::dna::capabilities::CapabilityCall;
    use test_utils;

    use holochain_core_types::{
        error::{DnaError, HolochainError},
        json::{JsonString, RawString},
    };

    /// dummy zome name compatible with ZomeFnCall
    pub fn test_zome() -> String {
        "test_zome".to_string()
    }

    /// dummy capability token
    pub fn test_capability_token() -> Address {
        Address::from(test_capability_token_str())
    }

    /// dummy capability token compatible with ZomeFnCall
    pub fn test_capability_token_str() -> String {
        "test_token".to_string()
    }

    /// dummy capability call
    pub fn test_capability_call() -> CapabilityCall {
        CapabilityCall::new(test_capability_token(), None)
    }

    /// dummy capability name compatible with ZomeFnCall
    pub fn test_capability_name() -> String {
        "test_cap".to_string()
    }

    /// dummy function name compatible with ZomeFnCall
    pub fn test_function() -> String {
        "test_function".to_string()
    }

    /// dummy parameters compatible with ZomeFnCall
    pub fn test_parameters() -> String {
        "".to_string()
    }

    /// dummy function call
    pub fn test_zome_call() -> ZomeFnCall {
        ZomeFnCall::new(
            &test_zome(),
            Some(test_capability_call()),
            &test_function(),
            test_parameters(),
        )
    }

    /// dummy function result
    pub fn test_call_response() -> ExecuteZomeFnResponse {
        ExecuteZomeFnResponse::new(test_zome_call(), Ok("foo".into()))
    }

    #[test]
    /// test the equality and uniqueness of function calls (based on internal snowflakes)
    fn test_zome_call_eq() {
        let zc1 = test_zome_call();
        let zc2 = test_zome_call();

        assert_eq!(zc1, zc1);
        assert_ne!(zc1, zc2);
    }

    #[test]
    /// test access to function result's function call
    fn test_zome_call_result() {
        let zome_call = test_zome_call();
        let call_result = ExecuteZomeFnResponse::new(zome_call.clone(), Ok("foo".into()));

        assert_eq!(call_result.call(), zome_call);
    }

    #[test]
    /// test access to the result of function result
    fn test_call_result_result() {
        assert_eq!(test_call_response().result(), Ok("foo".into()),);
    }

    #[test]
    /// smoke test the init of a nucleus
    fn can_instantiate_nucleus_state() {
        let nucleus_state = NucleusState::new();
        assert_eq!(nucleus_state.dna, None);
        assert_eq!(nucleus_state.has_initialized(), false);
        assert_eq!(nucleus_state.has_initialization_failed(), false);
        assert_eq!(nucleus_state.status(), NucleusStatus::New);
    }

    #[test]
    /// tests that calling a valid zome function returns a valid result
    fn test_call_zome_function() {
        let _netname = Some("test_call_zome_function");
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let (_, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");
        //let context = instance.initialize_context(test_context("janet", netname));

        // Create zome function call
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(test_capability_call()),
            "public_test_fn",
            "",
        );

        let result = context.block_on(call_zome_function(zome_call, &context));

        assert!(result.is_ok());
        assert_eq!(JsonString::from(RawString::from(1337)), result.unwrap());
    }

    #[test]
    /// tests that calling an invalid DNA returns the correct error
    fn call_ribosome_wrong_dna() {
        let netname = Some("call_ribosome_wrong_dna");
        let mut instance = Instance::new(test_context("janet", netname));
        let context = instance.initialize_without_dna(test_context("jane", netname));

        let call = ZomeFnCall::new(
            "test_zome",
            Some(test_capability_call()),
            "public_test_fn",
            "{}",
        );
        let result = context.block_on(call_zome_function(call, &context));

        match result {
            Err(HolochainError::DnaMissing) => {}
            _ => assert!(false),
        }
    }

    #[test]
    /// tests that calling a valid zome with invalid function returns the correct error
    fn call_ribosome_wrong_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let (_, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");

        // Create zome function call:
        let call = ZomeFnCall::new("test_zome", Some(test_capability_call()), "xxx", "{}");

        let result = context.block_on(call_zome_function(call, &context));

        match result {
            Err(HolochainError::Dna(DnaError::ZomeFunctionNotFound(err))) => {
                assert_eq!(err, "Zome function \'xxx\' not found in Zome 'test_zome'")
            }
            _ => assert!(false),
        }
    }

    #[test]
    /// tests that calling the wrong zome/capability returns the correct errors
    fn call_wrong_zome_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let (_, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");

        // Create bad zome function call
        let call = ZomeFnCall::new("xxx", Some(test_capability_call()), "public_test_fn", "{}");

        let result = context.block_on(call_zome_function(call, &context));

        match result {
            Err(HolochainError::Dna(err)) => assert_eq!(err.to_string(), "Zome 'xxx' not found"),
            _ => assert!(false),
        }

        /*
        convert when we actually have capabilities on a chain
                let mut cap_call = test_capability_call();
                cap_call.cap_name = "xxx".to_string();

                // Create bad capability function call
        let call = ZomeFnCall::new("test_zome", Some(cap_call), "public_test_fn", "{}");

                let result = super::call_and_wait_for_result(call, &mut instance);

                match result {
                    Err(HolochainError::Dna(err)) => assert_eq!(
                        err.to_string(),
                        "Capability 'xxx' not found in Zome 'test_zome'"
                    ),
                    _ => assert!(false),
                }
        */
    }

    #[test]
    fn test_zomefncall_same_as() {
        let base = ZomeFnCall::new("yoyo", Some(test_capability_call()), "fufu", "papa");
        let copy = ZomeFnCall::new("yoyo", Some(test_capability_call()), "fufu", "papa");
        let same = ZomeFnCall::new("yoyo", Some(test_capability_call()), "fufu", "papa1");
        let diff1 = ZomeFnCall::new("yoyo1", Some(test_capability_call()), "fufu", "papa");
        let diff2 = ZomeFnCall::new("yoyo", Some(test_capability_call()), "fufu3", "papa");

        assert_ne!(base, copy);
        assert!(base.same_fn_as(&copy));
        assert!(copy.same_fn_as(&base));
        assert!(base.same_fn_as(&same));
        assert!(!base.same_fn_as(&diff1));
        assert!(!base.same_fn_as(&diff2));
    }
}
