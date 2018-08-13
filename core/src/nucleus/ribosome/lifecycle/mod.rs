pub mod genesis;
pub mod validate_commit;
use action::ActionWrapper;
use error::HolochainError;
use hash_table::entry::Entry;
use holochain_dna::zome::capabilities::ReservedCapabilityNames;
use instance::Observer;
use nucleus::{
    call_zome_and_wait_for_result,
    ribosome::{
        lifecycle::{genesis::genesis, validate_commit::validate_commit},
        Defn,
    },
    FunctionCall,
};
use num_traits::FromPrimitive;
use std::{str::FromStr, sync::mpsc::Sender};

// Lifecycle functions are zome logic called by HC actions
// @TODO should each one be an action, e.g. Action::Genesis(Zome)?
// @see https://github.com/holochain/holochain-rust/issues/200

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum LifecycleFunction {
    /// Error index for unimplemented functions
    MissingNo = 0,

    /// LifeCycle Capability

    /// genesis() -> bool
    Genesis,

    /// validate_commit() -> bool
    ValidateCommit,

    /// Communication Capability

    /// receive(from : String, message : String) -> String
    Receive,
}

impl FromStr for LifecycleFunction {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "genesis" => Ok(LifecycleFunction::Genesis),
            "validate_commit" => Ok(LifecycleFunction::ValidateCommit),
            "receive" => Ok(LifecycleFunction::Receive),
            _ => Err("Cannot convert string to LifecycleFunction"),
        }
    }
}

impl LifecycleFunction {
    pub fn as_fn(
        &self,
    ) -> fn(
        action_channel: &Sender<ActionWrapper>,
        observer_channel: &Sender<Observer>,
        zome: &str,
        params: &LifecycleFunctionParams,
    ) -> LifecycleFunctionResult {
        fn noop(
            _action_channel: &Sender<ActionWrapper>,
            _observer_channel: &Sender<Observer>,
            _zome: &str,
            _params: &LifecycleFunctionParams,
        ) -> LifecycleFunctionResult {
            LifecycleFunctionResult::Pass
        }

        match *self {
            LifecycleFunction::MissingNo => noop,
            LifecycleFunction::Genesis => genesis,
            LifecycleFunction::ValidateCommit => validate_commit,
            // @TODO
            // @see https://github.com/holochain/holochain-rust/issues/201
            LifecycleFunction::Receive => noop,
        }
    }
}

impl Defn for LifecycleFunction {
    fn as_str(&self) -> &'static str {
        match *self {
            LifecycleFunction::MissingNo => "",
            LifecycleFunction::Genesis => "genesis",
            LifecycleFunction::ValidateCommit => "validate_commit",
            LifecycleFunction::Receive => "receive",
        }
    }

    fn str_index(s: &str) -> usize {
        match LifecycleFunction::from_str(s) {
            Ok(i) => i as usize,
            Err(_) => LifecycleFunction::MissingNo as usize,
        }
    }

    fn from_index(i: usize) -> Self {
        match FromPrimitive::from_usize(i) {
            Some(v) => v,
            None => LifecycleFunction::MissingNo,
        }
    }

    fn capabilities(&self) -> ReservedCapabilityNames {
        ReservedCapabilityNames::LifeCycle
    }
}

#[derive(Debug)]
pub enum LifecycleFunctionParams {
    Genesis,
    ValidateCommit(Entry),
}

impl ToString for LifecycleFunctionParams {
    fn to_string(&self) -> String {
        match self {
            LifecycleFunctionParams::Genesis => "".to_string(),
            LifecycleFunctionParams::ValidateCommit(entry) => entry.to_json(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LifecycleFunctionResult {
    Pass,
    Fail(String),
    NotImplemented,
}

pub fn call(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: &str,
    function: &LifecycleFunction,
    params: &LifecycleFunctionParams,
) -> LifecycleFunctionResult {
    let function_call = FunctionCall::new(
        zome,
        &function.capabilities().as_str().to_string(),
        &function.as_str().to_string(),
        &params.to_string(),
    );

    let call_result =
        call_zome_and_wait_for_result(function_call.clone(), &action_channel, &observer_channel);

    // translate the call result to a lifecycle result
    match call_result {
        // empty string OK = Success
        Ok(ref s) if s.is_empty() => LifecycleFunctionResult::Pass,

        // things that = NotImplemented
        Err(HolochainError::CapabilityNotFound(_)) => LifecycleFunctionResult::NotImplemented,
        Err(HolochainError::ZomeFunctionNotFound(_)) => LifecycleFunctionResult::NotImplemented,
        // @TODO this looks super fragile
        // without it we get stack overflows, but with it we rely on a specific string
        Err(HolochainError::ErrorGeneric(ref msg))
            if msg == &format!(
                "Function: Module doesn\'t have export {}_dispatch",
                function.as_str()
            ) =>
        {
            LifecycleFunctionResult::NotImplemented
        }

        // string value or error = fail
        Ok(s) => LifecycleFunctionResult::Fail(s),
        Err(err) => LifecycleFunctionResult::Fail(err.to_string()),
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    use holochain_dna::zome::capabilities::ReservedCapabilityNames;
    extern crate holochain_agent;
    extern crate wabt;
    use self::wabt::Wat2Wasm;
    use instance::{tests::test_instance, Instance};
    use nucleus::ribosome::lifecycle::LifecycleFunction;
    use std::str::FromStr;

    /// generates the wasm to dispatch any zome API function with a single memomry managed runtime
    /// and bytes argument
    pub fn test_lifecycle_function_wasm(canonical_name: &str, result: i32) -> Vec<u8> {
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
                // the signature is the same as the exported "test_dispatch" function below as
                // we want the latter to be a thin wrapper for the former
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
                // define and export the test_dispatch function that will be called from the
                // ribosome rust implementation, where "test" is the fourth arg to `call`
                // @see `test_zome_api_function_runtime`
                // @see nucleus::ribosome::call
                // (func (export "test_dispatch") ...)
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
        (export "{}_dispatch")
        (param $allocation i32)
        (result i32)

        (i32.const {})
    )
)
                "#,
                    canonical_name, result,
                ),
            )
            .unwrap()
            .as_ref()
            .to_vec()
    }

    pub fn test_lifecycle_function_instance(
        zome: &str,
        canonical_name: &str,
        result: i32,
    ) -> Instance {
        let dna = test_utils::create_test_dna_with_wasm(
            zome,
            ReservedCapabilityNames::LifeCycle.as_str(),
            test_lifecycle_function_wasm(canonical_name, result),
        );

        test_instance(dna)
    }

    #[test]
    /// test the FromStr implementation for LifecycleFunction
    fn test_from_str() {
        assert_eq!(
            LifecycleFunction::Genesis,
            LifecycleFunction::from_str("genesis").unwrap(),
        );
        assert_eq!(
            LifecycleFunction::ValidateCommit,
            LifecycleFunction::from_str("validate_commit").unwrap(),
        );
        assert_eq!(
            LifecycleFunction::Receive,
            LifecycleFunction::from_str("receive").unwrap(),
        );

        assert_eq!(
            "Cannot convert string to LifecycleFunction",
            LifecycleFunction::from_str("foo").unwrap_err(),
        );
    }

}
