use nucleus::FunctionCall;
use holochain_dna::zome::capabilities::ReservedCapabilityNames;
use nucleus::ribosome::lifecycle::LifecycleFunction;
use error::HolochainError;
use holochain_dna::zome::Zome;
use nucleus::call_zome_and_wait_for_result;
use std::sync::mpsc::Sender;
use action::ActionWrapper;
use instance::Observer;
use nucleus::return_initialization_result;
use nucleus::ribosome::Defn;

pub fn genesis(
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    zome: Zome,
) {

    // Make ExecuteZomeFunction Action for genesis()
    let call = FunctionCall::new(
        zome.name,
        ReservedCapabilityNames::LifeCycle.as_str().to_string(),
        LifecycleFunction::Genesis.as_str().to_string(),
        "".to_string(),
    );

    // Call Genesis and wait
    let call_result =
        call_zome_and_wait_for_result(call, &action_channel, &observer_channel);

    // genesis returns a string
    // "" == success, otherwise error value
    match call_result {
        // not okay if genesis returned an value
        Ok(ref s) if s != "" => {
            // Send a failed ReturnInitializationResult Action
            return_initialization_result(Some(s.to_string()), &action_channel);

            // Kill thread
            // TODO - Instead, Keep track of each zome's initialization.
            // @see https://github.com/holochain/holochain-rust/issues/78
            // Mark this one as failed and continue with other zomes
            return;
        }
        // its okay if hc_lifecycle or genesis not present
        Ok(_) | Err(HolochainError::CapabilityNotFound(_)) => { /* NA */ }
        Err(HolochainError::ErrorGeneric(ref msg))
            if msg == "Function: Module doesn\'t have export genesis_dispatch" =>
        { /* NA */ }
        // Init fails if something failed in genesis called
        Err(err) => {
            // TODO - Create test for this edge case
            // @see https://github.com/holochain/holochain-rust/issues/78
            // Send a failed ReturnInitializationResult Action
            return_initialization_result(Some(err.to_string()), &action_channel);

            // Kill thread
            // TODO - Instead, Keep track of each zome's initialization.
            // @see https://github.com/holochain/holochain-rust/issues/78
            // Mark this one as failed and continue with other zomes
            return;
        }
    }
}
