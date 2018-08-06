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
    let lifecycle_result = match call_result {
        // empty string OK = Success
        Ok(ref s) if s == "" => LifecycleFunctionResult::Pass,

        // things that = NotImplemented
        Err(HolochainError::CapabilityNotFound(_)) => LifecycleFunctionResult::NotImplemented,
        Err(HolochainError::ZomeFunctionNotFound(_)) => LifecycleFunctionResult::NotImplemented,

        // string value or error = fail
        Ok(s) => LifecycleFunctionResult::Fail(s),
        Err(err) => LifecycleFunctionResult::Fail(err.to_string()),
    };

    lifecycle_result

    // // genesis returns a string
    // // "" == success, otherwise error value
    // match call_result {
    //
    //     Ok(ref s) if s != "" => {
    //         // Send a failed ReturnInitializationResult Action
    //         return_initialization_result(Some(s.to_string()), &action_channel);
    //
    //         // Kill thread
    //         // TODO - Instead, Keep track of each zome's initialization.
    //         // @see https://github.com/holochain/holochain-rust/issues/78
    //         // Mark this one as failed and continue with other zomes
    //         return;
    //     }
    //     // its okay if hc_lifecycle or genesis not present
    //     Ok(_) | Err(HolochainError::CapabilityNotFound(_)) => { /* NA */ }
    //     Err(HolochainError::ErrorGeneric(ref msg))
    //         if msg == "Function: Module doesn\'t have export genesis_dispatch" =>
    //     { /* NA */ }
    //     // Init fails if something failed in genesis called
    //     Err(err) => {
    //         // TODO - Create test for this edge case
    //         // @see https://github.com/holochain/holochain-rust/issues/78
    //         // Send a failed ReturnInitializationResult Action
    //         return_initialization_result(Some(err.to_string()), &action_channel);
    //
    //         // Kill thread
    //         // TODO - Instead, Keep track of each zome's initialization.
    //         // @see https://github.com/holochain/holochain-rust/issues/78
    //         // Mark this one as failed and continue with other zomes
    //         return;
    //     }
    // }
}
