use super::call;
use action::ActionWrapper;
use instance::Observer;
use nucleus::ribosome::lifecycle::{
    LifecycleFunction, LifecycleFunctionParams, LifecycleFunctionResult,
};
use std::sync::mpsc::Sender;

pub fn validate_commit(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: &str,
    params: LifecycleFunctionParams,
) -> LifecycleFunctionResult {
    call(
        action_channel,
        observer_channel,
        zome,
        LifecycleFunction::ValidateCommit,
        params,
    )
}
