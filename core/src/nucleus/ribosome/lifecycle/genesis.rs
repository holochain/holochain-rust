use action::ActionWrapper;
use error::HolochainError;
use holochain_dna::zome::{capabilities::ReservedCapabilityNames, Zome};
use instance::Observer;
use nucleus::{
    call_zome_and_wait_for_result,
    ribosome::{lifecycle::LifecycleFunction, Defn},
    FunctionCall,
};
use std::sync::mpsc::Sender;
use nucleus::ribosome::lifecycle::LifecycleFunctionResult;

pub fn genesis(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: Zome,
    // we ignore params for genesis
    _params: &str,
) -> LifecycleFunctionResult {

    // Make ExecuteZomeFunction Action for genesis()
    let call = FunctionCall::new(
        zome.name,
        ReservedCapabilityNames::LifeCycle.as_str().to_string(),
        LifecycleFunction::Genesis.as_str().to_string(),
        "".to_string(),
    );

    // Call Genesis and wait
    let call_result = call_zome_and_wait_for_result(call, &action_channel, &observer_channel);

    // translate the call result to a lifecycle result
    match call_result {
        // empty string OK = Success
        Ok(ref s) if s.is_empty() => LifecycleFunctionResult::Pass,

        // things that = NotImplemented
        Err(HolochainError::CapabilityNotFound(_)) => LifecycleFunctionResult::NotImplemented,
        Err(HolochainError::ZomeFunctionNotFound(_)) => LifecycleFunctionResult::NotImplemented,
        Err(HolochainError::ErrorGeneric(ref msg)) if msg == "Function: Module doesn\'t have export genesis_dispatch" => LifecycleFunctionResult::NotImplemented,

        // string value or error = fail
        Ok(s) => LifecycleFunctionResult::Fail(s),
        Err(err) => LifecycleFunctionResult::Fail(err.to_string()),
    }

}
