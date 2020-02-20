//! Module for ZomeCallbacks
//! ZomeCallbacks are functions in a Zome that are callable by the ribosome.

pub mod init;
pub mod links_utils;
pub mod receive;
pub mod validation_package;

use crate::{
    context::Context,
    nucleus::CallbackFnCall,
    wasm_engine::{
        self,
        callback::{init::init, receive::receive},
        runtime::WasmCallData,
        Defn,
    },
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{entry::Entry, validation::ValidationPackageDefinition};

use holochain_json_api::{
    error::JsonError,
    json::{default_to_json, JsonString},
};

use holochain_wasm_utils::api_serialization::receive::ReceiveParams;
use num_traits::FromPrimitive;
use serde_json;
use std::{str::FromStr, sync::Arc};

/// Enumeration of all Zome Callbacks known and used by Holochain
/// Enumeration can convert to str
// @TODO should each one be an action, e.g. Action::Init(Zome)?
// @see https://github.com/holochain/holochain-rust/issues/200

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum Callback {
    /// Error index for unimplemented functions
    MissingNo = 0,

    /// init() -> bool
    Init,

    /// receive(from: Address, message: String) -> String
    Receive,
}

impl FromStr for Callback {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init" => Ok(Callback::Init),
            "receive" => Ok(Callback::Receive),
            other if other.is_empty() => Ok(Callback::MissingNo),
            _ => Err("Cannot convert string to Callback"),
        }
    }
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Callback {
    // cannot test this because PartialEq is not implemented for fns
    #[cfg_attr(tarpaulin, skip)]
    pub fn as_fn(
        &self,
    ) -> fn(context: Arc<Context>, zome: &str, params: &CallbackParams) -> CallbackResult {
        fn noop(_context: Arc<Context>, _zome: &str, _params: &CallbackParams) -> CallbackResult {
            CallbackResult::Pass
        }

        match *self {
            Callback::MissingNo => noop,
            Callback::Init => init,
            // @TODO call this from somewhere
            // @see https://github.com/holochain/holochain-rust/issues/201
            Callback::Receive => receive,
        }
    }
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Defn for Callback {
    fn as_str(&self) -> &'static str {
        match *self {
            Callback::MissingNo => "",
            Callback::Init => "init",
            Callback::Receive => "receive",
        }
    }

    fn str_to_index(s: &str) -> usize {
        match Callback::from_str(s) {
            Ok(i) => i as usize,
            Err(_) => Callback::MissingNo as usize,
        }
    }

    fn from_index(i: usize) -> Self {
        match FromPrimitive::from_usize(i) {
            Some(v) => v,
            None => Callback::MissingNo,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson)]
#[allow(clippy::large_enum_variant)]
pub enum CallbackParams {
    Init,
    ValidateCommit(Entry),
    Receive(ReceiveParams),
}

impl ToString for CallbackParams {
    fn to_string(&self) -> String {
        match self {
            CallbackParams::Init => String::new(),
            CallbackParams::ValidateCommit(serialized_entry) => {
                String::from(JsonString::from(serialized_entry.to_owned()))
            }
            CallbackParams::Receive(params) => JsonString::from(params).to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CallbackResult {
    Pass,
    Fail(String),
    NotImplemented(String),
    ValidationPackageDefinition(ValidationPackageDefinition),
    ReceiveResult(String),
}

impl From<CallbackResult> for JsonString {
    fn from(v: CallbackResult) -> Self {
        default_to_json(v)
    }
}

impl From<JsonString> for CallbackResult {
    fn from(json_string: JsonString) -> CallbackResult {
        let r#try: Result<CallbackResult, serde_json::Error> =
            serde_json::from_str(&String::from(json_string.clone()));
        match r#try {
            Ok(callback_result) => callback_result,
            Err(_) => CallbackResult::Fail(String::from(json_string)),
        }
    }
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub(crate) fn run_callback(context: Arc<Context>, call: CallbackFnCall) -> CallbackResult {
    match wasm_engine::run_dna(
        WasmCallData::new_callback_call(context, call),
        Some(call.clone().parameters),
    ) {
        Ok(call_result) => {
            if call_result.is_null() {
                CallbackResult::Pass
            } else {
                CallbackResult::Fail(call_result.to_string())
            }
        }
        Err(_) => CallbackResult::NotImplemented("run_callback".into()),
    }
}

#[autotrace]
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn call(
    context: Arc<Context>,
    zome: &str,
    function: &Callback,
    params: &CallbackParams,
) -> CallbackResult {
    let call = CallbackFnCall::new(zome, &function.as_str().to_string(), (*params).clone());
    let dna = context.get_dna().expect("Callback called without DNA set!");
    match dna.get_wasm_from_zome_name(zome) {
        None => CallbackResult::NotImplemented("call/1".into()),
        Some(wasm) => {
            if wasm.code.is_empty() {
                CallbackResult::NotImplemented("call/2".into())
            } else {
                run_callback(context, call)
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use self::wabt::Wat2Wasm;
    use crate::{
        instance::{tests::test_instance, Instance},
        wasm_engine::{callback::Callback, Defn},
    };
    use test_utils;
    use wabt;

    /// generates the wasm to dispatch any zome API function with a single memomry managed runtime
    /// and bytes argument
    pub fn test_callback_wasm(canonical_name: &str, result: u64) -> Vec<u8> {
        Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(
                // We don't expect everyone to be a pro at hand-coding WAT so here's a "how to".
                // WAT does not have comments so code is duplicated in the comments here.
                //
                // How this works:
                //
                // root of the s-expression tree
                // (module ...)
                //
                // imports must be the first expressions in a module
                // imports the fn from the rust environment using its canonical zome API function
                // name as the function named `$zome_api_function` in WAT
                // define the signature as 1 input, 1 output
                // (import "env" "<canonical name>"
                //      (func $zome_api_function
                //          (param i64)
                //          (result i64)
                //      )
                // )
                //
                // only need 1 page of memory for testing
                // (memory 1)
                //
                // all modules compiled with rustc must have an export named "memory" (or fatal)
                // (export "memory" (memory 0))
                //
                // define and export the test function that will be called from the
                // ribosome rust implementation, where "test" is the fourth arg to `call`
                // @see `test_zome_api_function_runtime`
                // @see nucleus::ribosome::call
                // (func (export "test") ...)
                //
                // define the memory allocation for the memory manager that the serialized input
                // struct can be found across as an i64 to the exported function, also the function
                // return type is i64
                // (param $allocation i64)
                // (result i64)
                //
                // call the imported function and pass the exported function arguments straight
                // through, let the return also fall straight through
                // `get_local` maps the relevant arguments in the local scope
                // (call
                //      $zome_api_function
                //      (get_local $allocation)
                // )
                format!(
                    r#"
(module

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "{}")
        (param $allocation i64)
        (result i64)

        (i64.const {})
    )
)
                "#,
                    canonical_name, result,
                ),
            )
            .expect("string literal should be valid WAT")
            .as_ref()
            .to_vec()
    }

    pub fn test_callback_instance(
        zome: &str,
        canonical_name: &str,
        result: u64,
        network_name: Option<&str>,
    ) -> Result<Instance, String> {
        let dna =
            test_utils::create_test_dna_with_wasm(zome, test_callback_wasm(canonical_name, result));
        test_instance(dna, network_name)
    }

    #[test]
    /// test the FromStr implementation for Lifecycle Function
    fn test_from_str() {
        assert_eq!(
            Callback::Init,
            Callback::from_str("init").expect("string literal should be valid callback")
        );
        assert_eq!(
            Callback::Receive,
            Callback::from_str("receive").expect("string literal should be valid callback")
        );

        assert_eq!(
            "Cannot convert string to Callback",
            Callback::from_str("foo").expect_err("string literal shouldn't be valid callback"),
        );
    }

    #[test]
    fn defn_test() {
        // as_str()
        for (input, output) in vec![
            (Callback::MissingNo, ""),
            (Callback::Init, "init"),
            (Callback::Receive, "receive"),
        ] {
            assert_eq!(output, input.as_str());
        }

        // str_to_index()
        for (input, output) in vec![("", 0), ("init", 1), ("receive", 2)] {
            assert_eq!(output, Callback::str_to_index(input));
        }

        // from_index()
        for (input, output) in vec![
            (0, Callback::MissingNo),
            (1, Callback::Init),
            (2, Callback::Receive),
        ] {
            assert_eq!(output, Callback::from_index(input));
        }
    }
}
