use action::ActionWrapper;
use holochain_dna::zome::Zome;
use instance::Observer;
use std::sync::mpsc::Sender;
use nucleus::ribosome::lifecycle::LifecycleFunctionResult;

pub fn validate_commit(
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
    _zome: Zome,
) -> LifecycleFunctionResult {
    LifecycleFunctionResult::Pass
}
