pub mod genesis;
pub mod validate_commit;
use action::ActionWrapper;
use holochain_dna::zome::Zome;
use instance::Observer;
use nucleus::ribosome::{
    lifecycle::{genesis::genesis, validate_commit::validate_commit},
    Defn,
};
use num_traits::FromPrimitive;
use std::{str::FromStr, sync::mpsc::Sender};

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
    ) -> fn(action_channel: &Sender<ActionWrapper>, observer_channel: &Sender<Observer>, zome: Zome) -> LifecycleFunctionResult
    {
        fn noop(
            _action_channel: &Sender<ActionWrapper>,
            _observer_channel: &Sender<Observer>,
            _zome: Zome,
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
}

#[derive(Clone)]
pub enum LifecycleFunctionResult {
    Pass,
    Fail(String),
    NotImplemented,
}
