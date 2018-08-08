pub mod genesis;
pub mod validate_commit;
use action::ActionWrapper;
use instance::Observer;
use nucleus::ribosome::{
    lifecycle::{genesis::genesis, validate_commit::validate_commit},
    Defn,
};
use hash_table::entry::Entry;
use num_traits::FromPrimitive;
use std::{str::FromStr, sync::mpsc::Sender};
use nucleus::FunctionCall;
use error::HolochainError;
use nucleus::call_zome_and_wait_for_result;
use holochain_dna::zome::capabilities::ReservedCapabilityNames;

// Lifecycle functions are zome logic called by HC actions
// @TODO should each one be an action, e.g. Action::Genesis(Zome)?

#[derive(FromPrimitive)]
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
            "validate_commit" => Ok(LifecycleFunction::Genesis),
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
        params: LifecycleFunctionParams) -> LifecycleFunctionResult
    {
        fn noop(
            _action_channel: &Sender<ActionWrapper>,
            _observer_channel: &Sender<Observer>,
            _zome: &str,
            _params: LifecycleFunctionParams,
        ) -> LifecycleFunctionResult {
            LifecycleFunctionResult::Pass
        }

        match *self {
            LifecycleFunction::MissingNo => noop,
            LifecycleFunction::Genesis => genesis,
            LifecycleFunction::ValidateCommit => validate_commit,
            // @TODO
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

#[derive(Clone, Debug)]
pub enum LifecycleFunctionResult {
    Pass,
    Fail(String),
    NotImplemented,
}

pub fn call(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: &str,
    function: LifecycleFunction,
    params: LifecycleFunctionParams,
) -> LifecycleFunctionResult {

    let function_call = FunctionCall::new(
        zome,
        &function.capabilities().as_str().to_string(),
        &function.as_str().to_string(),
        &params.to_string(),
    );

    let call_result = call_zome_and_wait_for_result(function_call.clone(), &action_channel, &observer_channel);

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
            if msg == &format!("Function: Module doesn\'t have export {}_dispatch", function.as_str()) =>
            LifecycleFunctionResult::NotImplemented,

        // string value or error = fail
        Ok(s) => LifecycleFunctionResult::Fail(s),
        // Err(err) => LifecycleFunctionResult::Fail(err.to_string()),
        _ => LifecycleFunctionResult::Pass,
    }

}
