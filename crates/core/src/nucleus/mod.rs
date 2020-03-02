/// Nucleus is the module that handles DNA, including the Ribosome.
///
pub mod actions;
pub mod reducers;
pub mod state;
pub mod validation;
pub use crate::{
    context::Context,
    nucleus::{
        actions::call_zome_function::{
            call_zome_function, make_cap_request_for_call, ExecuteZomeFnResponse,
        },
        reducers::reduce,
        state::ZomeFnCallState,
    },
};
use holochain_core_types::{dna::capabilities::CapabilityRequest, error::HcResult};
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::Address;
use snowflake;
use std::sync::Arc;

/// Struct holding data for tracing the call of an HDK function from a zome function
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct WasmApiFnCall {
    /// imports! name of the function that is invoked by the wasm guest
    pub function: String,
    pub parameters: JsonString,
}

pub type WasmApiFnCallResult = Result<JsonString, String>;

/// Struct holding data for requesting the execution of a Zome function (QueueZomeFunctionCall Action)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct ZomeFnCall {
    id: snowflake::ProcessUniqueId,
    pub zome_name: String,
    pub cap: CapabilityRequest,
    pub fn_name: String,
    pub parameters: JsonString,
}

impl ZomeFnCall {
    pub fn new<J: Into<JsonString>>(
        zome: &str,
        cap: CapabilityRequest,
        function: &str,
        parameters: J,
    ) -> Self {
        ZomeFnCall {
            // @TODO can we defer to the ActionWrapper id?
            // @see https://github.com/holochain/holochain-rust/issues/198
            id: snowflake::ProcessUniqueId::new(),
            zome_name: zome.to_string(),
            cap,
            fn_name: function.to_string(),
            parameters: parameters.into(),
        }
    }

    pub fn create<J: Into<JsonString>>(
        context: Arc<Context>,
        zome: &str,
        token: Address,
        function: &str,
        parameters: J,
    ) -> Self {
        let params = parameters.into();
        ZomeFnCall::new(
            zome,
            make_cap_request_for_call(context, token, function, params.clone()),
            function,
            params,
        )
    }

    pub fn same_fn_as(&self, fn_call: &ZomeFnCall) -> bool {
        self.zome_name == fn_call.zome_name
            && self.cap == fn_call.cap
            && self.fn_name == fn_call.fn_name
    }

    pub fn cap_token(&self) -> Address {
        self.cap.cap_token.clone()
    }

    pub fn id(&self) -> snowflake::ProcessUniqueId {
        self.id
    }
}

pub type ZomeFnResult = HcResult<JsonString>;

/// Struct holding data for requesting the execution of a callback function
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallbackFnCall {
    id: snowflake::ProcessUniqueId,
    pub zome_name: String,
    pub fn_name: String,
    pub parameters: JsonString,
}

impl CallbackFnCall {
    pub fn new<J: Into<JsonString>>(zome: &str, function: &str, parameters: J) -> Self {
        CallbackFnCall {
            // @TODO can we defer to the ActionWrapper id?
            // @see https://github.com/holochain/holochain-rust/issues/198
            id: snowflake::ProcessUniqueId::new(),
            zome_name: zome.to_string(),
            fn_name: function.to_string(),
            parameters: parameters.into(),
        }
    }
}

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
        wasm_engine::api::call::tests::setup_test,
    };
    use test_utils;

    use holochain_core_types::{
        dna::capabilities::CapabilityRequest,
        error::{DnaError, HolochainError},
        signature::Signature,
    };
    use holochain_json_api::json::{JsonString, RawString};
    use holochain_persistence_api::cas::content::AddressableContent;

    /// dummy zome name compatible with ZomeFnCall
    pub fn test_zome() -> String {
        "test_zome".to_string()
    }

    /// dummy capability token
    pub fn dummy_capability_token() -> Address {
        Address::from(dummy_capability_token_str())
    }

    /// dummy capability token
    pub fn dummy_caller() -> Address {
        Address::from(dummy_caller_str())
    }

    /// dummy capability token compatible with ZomeFnCall
    pub fn dummy_capability_token_str() -> String {
        "dummy_token".to_string()
    }

    /// dummy capability caller compatible with ZomeFnCall
    pub fn dummy_caller_str() -> String {
        "dummy_caller".to_string()
    }

    /// test capability call
    pub fn test_capability_request<J: Into<JsonString>>(
        context: Arc<Context>,
        function: &str,
        parameters: J,
    ) -> CapabilityRequest {
        make_cap_request_for_call(
            context.clone(),
            dummy_capability_token(),
            function,
            parameters,
        )
    }

    /// test self agent capability call
    pub fn test_agent_capability_request<J: Into<JsonString>>(
        context: Arc<Context>,
        function: &str,
        parameters: J,
    ) -> CapabilityRequest {
        make_cap_request_for_call(
            context.clone(),
            Address::from(context.agent_id.address()),
            function,
            parameters,
        )
    }
    /// dummy capability call
    pub fn dummy_capability_request() -> CapabilityRequest {
        CapabilityRequest::new(
            dummy_capability_token(),
            Address::from("test caller"),
            Signature::fake(),
        )
    }

    /// dummy capability name compatible with ZomeFnCall
    pub fn test_capability_name() -> String {
        "hc_public".to_string()
    }

    /// dummy function name compatible with ZomeFnCall
    pub fn test_function() -> String {
        "test_function".to_string()
    }

    /// dummy parameters compatible with ZomeFnCall
    pub fn test_parameters() -> JsonString {
        JsonString::empty_object()
    }

    /// dummy function call
    pub fn test_zome_call() -> ZomeFnCall {
        ZomeFnCall::new(
            &test_zome(),
            dummy_capability_request(),
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
        assert_eq!(nucleus_state.initialization().is_some(), false);
        assert_eq!(nucleus_state.has_initialization_failed(), false);
        assert_eq!(nucleus_state.status(), NucleusStatus::New);
    }

    #[test]
    /// tests that calling a valid zome function returns a valid result
    fn test_call_zome_function() {
        let _netname = Some("test_call_zome_function");
        let dna = test_utils::create_test_dna_with_wat("test_zome", None);
        //let (_instance, context) =
        //    test_instance_and_context(dna, None).expect("Could not initialize test instance");
        //let context = instance.initialize_context(test_context("janet", netname));
        let test_setup = setup_test(dna, "test_call_zome_function");
        let context = test_setup.context.clone();
        let token = context.get_public_token().unwrap();

        // Create zome function call
        let zome_call =
            ZomeFnCall::create(context.clone(), "test_zome", token, "public_test_fn", "");

        let result = context.block_on(call_zome_function(Arc::clone(&context), zome_call));

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
            dummy_capability_request(),
            "public_test_fn",
            "{}",
        );
        let result = context.block_on(call_zome_function(Arc::clone(&context), call));

        match result {
            Err(HolochainError::DnaMissing) => {}
            _ => assert!(false),
        }
    }

    #[test]
    /// tests that calling a valid zome with invalid function returns the correct error
    fn call_ribosome_wrong_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", None);
        let (_instance, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");

        // Create zome function call:
        let call = ZomeFnCall::new("test_zome", dummy_capability_request(), "xxx", "{}");

        let result = context.block_on(call_zome_function(Arc::clone(&context), call));

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
        let dna = test_utils::create_test_dna_with_wat("test_zome", None);
        let (_instance, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");

        // Create bad zome function call
        let call = ZomeFnCall::new("xxx", dummy_capability_request(), "public_test_fn", "{}");

        let result = context.block_on(call_zome_function(Arc::clone(&context), call));

        match result {
            Err(HolochainError::Dna(err)) => assert_eq!(err.to_string(), "Zome 'xxx' not found"),
            _ => assert!(false),
        }

        /*
        convert when we actually have capabilities on a chain
                let mut cap_call = test_capability_request();
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
        let base = ZomeFnCall::new("yoyo", dummy_capability_request(), "fufu", "papa");
        let copy = ZomeFnCall::new("yoyo", dummy_capability_request(), "fufu", "papa");
        let same = ZomeFnCall::new("yoyo", dummy_capability_request(), "fufu", "papa1");
        let diff1 = ZomeFnCall::new("yoyo1", dummy_capability_request(), "fufu", "papa");
        let diff2 = ZomeFnCall::new("yoyo", dummy_capability_request(), "fufu3", "papa");

        assert_ne!(base, copy);
        assert!(base.same_fn_as(&copy));
        assert!(copy.same_fn_as(&base));
        assert!(base.same_fn_as(&same));
        assert!(!base.same_fn_as(&diff1));
        assert!(!base.same_fn_as(&diff2));
    }
}
