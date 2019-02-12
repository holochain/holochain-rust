use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        ribosome::{self, api::call::validate_call},
        state::NucleusState,
        ZomeFnCall, ZomeFnResult,
    },
};
use holochain_core_types::{
    dna::{
        wasm::DnaWasm,
    },
    error::{HolochainError},
    json::JsonString,
};
use std::{
    sync::{
        mpsc::{SyncSender},
        Arc,
    },
    thread,
};

/// Reduce ExecuteZomeFunction Action
/// Execute an exposed Zome function in a separate thread and send the result in
/// a ReturnZomeFunctionResult Action on success or failure
pub fn reduce_execute_zome_function(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    let fn_call = match action_wrapper.action().clone() {
        Action::ExecuteZomeFunction(call) => call,
        _ => unreachable!(),
    };

    fn dispatch_error_result(
        action_channel: &SyncSender<ActionWrapper>,
        fn_call: &ZomeFnCall,
        error: HolochainError,
    ) {
        let zome_not_found_response =
            ExecuteZomeFnResponse::new(fn_call.clone(), Err(error.clone()));

        action_channel
            .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(
                zome_not_found_response,
            )))
            .expect("action channel to be open in reducer");
    }

    // 1. Validate the call (a number of things could go wrong)
    let dna = match validate_call(context.clone(), state, &fn_call) {
        Err(err) => {
            // Notify failure
            dispatch_error_result(context.action_channel(), &fn_call, err);
            return;
        }
        Ok(dna) => dna,
    };

    // 2. function WASM and execute it in a separate thread
    let maybe_code = dna.get_wasm_from_zome_name(fn_call.zome_name.clone());
    let code =
        maybe_code.expect("zome not found, Should have failed before when getting capability.");

    // Ok Zome function is defined in given capability.
    // Prepare call - FIXME is this really useful?
    state.zome_calls.insert(fn_call.clone(), None);
    // Launch thread with function call
    launch_zome_fn_call(context, fn_call, &code, state.dna.clone().unwrap().name);
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ExecuteZomeFnResponse {
    call: ZomeFnCall,
    result: ZomeFnResult,
}

impl ExecuteZomeFnResponse {
    pub fn new(call: ZomeFnCall, result: Result<JsonString, HolochainError>) -> Self {
        ExecuteZomeFnResponse { call, result }
    }

    /// read only access to call
    pub fn call(&self) -> ZomeFnCall {
        self.call.clone()
    }

    /// read only access to result
    pub fn result(&self) -> Result<JsonString, HolochainError> {
        self.result.clone()
    }
}

pub(crate) fn launch_zome_fn_call(
    context: Arc<Context>,
    zome_call: ZomeFnCall,
    wasm: &DnaWasm,
    dna_name: String,
) {
    let code = wasm.code.clone();

    thread::spawn(move || {
        // Have Ribosome spin up DNA and call the zome function
        let call_result = ribosome::run_dna(
            &dna_name,
            context.clone(),
            code,
            &zome_call,
            Some(zome_call.clone().parameters.into_bytes()),
        );
        // Construct response
        let response = ExecuteZomeFnResponse::new(zome_call.clone(), call_result);
        // Send ReturnZomeFunctionResult Action
        context
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(
                response,
            )))
            .expect("action channel to be open in reducer");
    });
}





#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    use super::*;
    use crate::{
        action::ActionWrapper,
        instance::{
            tests::test_context_with_channels,
            Observer,
        },
        nucleus::{
            reduce,
            state::NucleusState,
            tests::test_capability_call,
        },
    };
    use std::sync::{mpsc::sync_channel, Arc};


    #[test]
    /// smoke test reducing over a nucleus
    fn can_reduce_execfn_action() {
        let call = ZomeFnCall::new("myZome", Some(test_capability_call()), "bogusfn", "");

        let action_wrapper = ActionWrapper::new(Action::ExecuteZomeFunction(call));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = sync_channel::<ActionWrapper>(10);
        let (tx_observer, _observer) = sync_channel::<Observer>(10);
        let context = test_context_with_channels("jimmy", &sender, &tx_observer, None);

        let reduced_nucleus = reduce(context, nucleus.clone(), &action_wrapper);
        assert_eq!(nucleus, reduced_nucleus);
    }
}