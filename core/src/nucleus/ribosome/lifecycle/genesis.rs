use action::ActionWrapper;
use instance::Observer;
use nucleus::{
    ribosome::{lifecycle::LifecycleFunction},
};
use std::sync::mpsc::Sender;
use nucleus::ribosome::lifecycle::LifecycleFunctionResult;
use nucleus::ribosome::lifecycle::LifecycleFunctionParams;
use super::call;

pub fn genesis(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: &str,
    // we ignore params for genesis
    params: LifecycleFunctionParams,
) -> LifecycleFunctionResult {

    call(
        action_channel,
        observer_channel,
        zome,
        LifecycleFunction::Genesis,
        params,
    )

}
