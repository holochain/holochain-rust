//! Module for ZomeCallbacks
//! ZomeCallbacks are functions in a Zome that are callable by the ribosome.

pub mod genesis;
pub mod receive;
pub mod validate_commit;

use action::ActionWrapper;
use context::Context;
use hash_table::entry::Entry;
use holochain_dna::{wasm::DnaWasm, zome::capabilities::ReservedCapabilityNames};
use instance::Observer;
use json::ToJson;
use nucleus::{
    ribosome::{
        self,
        callback::{genesis::genesis, receive::receive, validate_commit::validate_commit},
        Defn,
    },
    ZomeFnCall,
};
use num_traits::FromPrimitive;
use std::{
    str::FromStr,
    sync::{mpsc::Sender, Arc},
    thread::sleep,
    time::Duration,
};

/// Enumeration of all Zome Callbacks known and used by Holochain
/// Enumeration can convert to str
// @TODO should each one be an action, e.g. Action::Genesis(Zome)?
// @see https://github.com/holochain/holochain-rust/issues/200

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum Callback {
    /// Error index for unimplemented functions
    MissingNo = 0,

    /// MissingNo Capability

    /// validate_commit() -> bool
    ValidateCommit,

    /// LifeCycle Capability

    /// genesis() -> bool
    Genesis,

    /// Communication Capability

    /// receive(from: String, message: String) -> String
    Receive,
}

impl FromStr for Callback {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "genesis" => Ok(Callback::Genesis),
            "validate_commit" => Ok(Callback::ValidateCommit),
            "receive" => Ok(Callback::Receive),
            "" => Ok(Callback::MissingNo),
            _ => Err("Cannot convert string to Callback"),
        }
    }
}

impl Callback {
    pub fn as_fn(
        &self,
    ) -> fn(
        context: Arc<Context>,
        action_channel: &Sender<ActionWrapper>,
        observer_channel: &Sender<Observer>,
        zome: &str,
        params: &CallbackParams,
    ) -> CallbackResult {
        fn noop(
            _context: Arc<Context>,
            _action_channel: &Sender<ActionWrapper>,
            _observer_channel: &Sender<Observer>,
            _zome: &str,
            _params: &CallbackParams,
        ) -> CallbackResult {
            CallbackResult::Pass
        }

        match *self {
            Callback::MissingNo => noop,
            Callback::Genesis => genesis,
            Callback::ValidateCommit => validate_commit,
            // @TODO call this from somewhere
            // @see https://github.com/holochain/holochain-rust/issues/201
            Callback::Receive => receive,
        }
    }
}

impl Defn for Callback {
    fn as_str(&self) -> &'static str {
        match *self {
            Callback::MissingNo => "",
            Callback::Genesis => "genesis",
            Callback::ValidateCommit => "validate_commit",
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

    fn capability(&self) -> ReservedCapabilityNames {
        match *self {
            Callback::MissingNo => ReservedCapabilityNames::MissingNo,
            Callback::Genesis => ReservedCapabilityNames::LifeCycle,
            // @TODO needs a sensible capability
            // @see https://github.com/holochain/holochain-rust/issues/133
            Callback::ValidateCommit => ReservedCapabilityNames::MissingNo,
            // @TODO call this from somewhere
            // @see https://github.com/holochain/holochain-rust/issues/201
            Callback::Receive => ReservedCapabilityNames::Communication,
        }
    }
}

#[derive(Debug)]
pub enum CallbackParams {
    Genesis,
    ValidateCommit(Entry),
    // @TODO call this from somewhere
    // @see https://github.com/holochain/holochain-rust/issues/201
    Receive,
}

impl ToString for CallbackParams {
    fn to_string(&self) -> String {
        match self {
            CallbackParams::Genesis => "".to_string(),
            CallbackParams::ValidateCommit(entry) => entry.to_json().unwrap_or_default(),
            CallbackParams::Receive => "".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CallbackResult {
    Pass,
    Fail(String),
    NotImplemented,
}

pub(crate) fn run_callback(
    context: Arc<Context>,
    fc: ZomeFnCall,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    wasm: &DnaWasm,
    app_name: String,
) -> CallbackResult {
    match ribosome::api::call(
        &app_name,
        context,
        &action_channel,
        &observer_channel,
        wasm.code.clone(),
        &fc,
        Some(fc.clone().parameters.into_bytes()),
    ) {
        Ok(runtime) => match runtime.result.is_empty() {
            true => CallbackResult::Pass,
            false => CallbackResult::Fail(runtime.result),
        },
        Err(_) => CallbackResult::NotImplemented,
    }
}

pub fn call(
    context: Arc<Context>,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: &str,
    function: &Callback,
    params: &CallbackParams,
) -> CallbackResult {
    let zome_call = ZomeFnCall::new(
        zome,
        &function.capability().as_str().to_string(),
        &function.as_str().to_string(),
        &params.to_string(),
    );

    // In the case of genesis we encounter race conditions with regards to setting the DNA.
    // Genesis gets called asynchronously right after dispatching an action that sets the DNA in
    // the state, which can result in this code being executed first.
    // But we can't run anything if there is no DNA which holds the WASM, so we have to wait here.
    // TODO: use a future here
    let mut dna = None;
    let mut done = false;
    let mut tries = 0;
    while !done {
        {
            let state = context
                .state()
                .expect("Callback called without application state!");
            dna = state.nucleus().dna();
        }
        match dna {
            Some(_) => done = true,
            None => {
                if tries > 10 {
                    done = true;
                } else {
                    sleep(Duration::from_millis(10));
                    tries += 1;
                }
            }
        }
    }

    let dna = dna.expect("Callback called without DNA set!");

    match dna.get_wasm_from_zome_name(zome) {
        None => CallbackResult::NotImplemented,
        Some(wasm) => {
            if wasm.code.is_empty() {
                CallbackResult::NotImplemented
            } else {
                run_callback(
                    context.clone(),
                    zome_call,
                    action_channel,
                    observer_channel,
                    wasm,
                    dna.name.clone(),
                )
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    extern crate holochain_agent;
    extern crate test_utils;
    extern crate wabt;
    use self::wabt::Wat2Wasm;
    use instance::{tests::test_instance, Instance};
    use nucleus::ribosome::{callback::Callback, Defn};
    use std::str::FromStr;

    /// generates the wasm to dispatch any zome API function with a single memomry managed runtime
    /// and bytes argument
    pub fn test_callback_wasm(canonical_name: &str, result: i32) -> Vec<u8> {
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
                //          (param i32)
                //          (result i32)
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
                // struct can be found across as an i32 to the exported function, also the function
                // return type is i32
                // (param $allocation i32)
                // (result i32)
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
        (param $allocation i32)
        (result i32)

        (i32.const {})
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

    pub fn test_callback_instance(zome: &str, canonical_name: &str, result: i32) -> Instance {
        let dna = test_utils::create_test_dna_with_wasm(
            zome,
            Callback::from_str(canonical_name)
                .expect("string argument canonical_name should be valid callback")
                .capability()
                .as_str(),
            test_callback_wasm(canonical_name, result),
        );

        test_instance(dna)
    }

    #[test]
    /// test the FromStr implementation for Lifecycle Function
    fn test_from_str() {
        assert_eq!(
            Callback::Genesis,
            Callback::from_str("genesis").expect("string literal should be valid callback")
        );
        assert_eq!(
            Callback::ValidateCommit,
            Callback::from_str("validate_commit").expect("string literal should be valid callback"),
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

}
