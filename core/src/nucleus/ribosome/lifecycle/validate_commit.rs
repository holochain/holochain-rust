use action::ActionWrapper;
use instance::Observer;
use std::sync::mpsc::Sender;
use nucleus::ribosome::lifecycle::LifecycleFunction;
use nucleus::ribosome::lifecycle::LifecycleFunctionResult;
use nucleus::ribosome::lifecycle::LifecycleFunctionParams;
use super::call;

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
