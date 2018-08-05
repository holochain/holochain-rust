use std::sync::mpsc::Sender;
use action::ActionWrapper;
use instance::Observer;
use holochain_dna::zome::Zome;

pub fn validate_commit(
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
    _zome: Zome,
) {

}
