use action::ActionWrapper;
use holochain_dna::zome::Zome;
use instance::Observer;
use std::sync::mpsc::Sender;
use nucleus::ribosome::lifecycle::LifecycleFunction;
use nucleus::ribosome::lifecycle::LifecycleFunctionResult;
use holochain_dna::zome::capabilities::ReservedCapabilityNames;
use nucleus::FunctionCall;
use nucleus::ribosome::Defn;
use error::HolochainError;
use nucleus::call_zome_and_wait_for_result;

pub fn validate_commit(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: Zome,
    params: &str,
) -> LifecycleFunctionResult {

    let call = FunctionCall::new(
        zome.name,
        ReservedCapabilityNames::LifeCycle.as_str().to_string(),
        LifecycleFunction::ValidateCommit.as_str().to_string(),
        // ignore params for genesis
        params.to_string(),
    );

    let call_result = call_zome_and_wait_for_result(call, &action_channel, &observer_channel);

    // translate the call result to a lifecycle result
    match call_result {
        // empty string OK = Success
        Ok(ref s) if s.is_empty() => LifecycleFunctionResult::Pass,

        // things that = NotImplemented
        Err(HolochainError::CapabilityNotFound(_)) => LifecycleFunctionResult::NotImplemented,
        Err(HolochainError::ZomeFunctionNotFound(_)) => LifecycleFunctionResult::NotImplemented,
        Err(HolochainError::ErrorGeneric(ref msg)) if msg == "Function: Module doesn\'t have export validate_commit_dispatch" => LifecycleFunctionResult::NotImplemented,

        // string value or error = fail
        Ok(s) => LifecycleFunctionResult::Fail(s),
        Err(err) => LifecycleFunctionResult::Fail(err.to_string()),
    }

}
