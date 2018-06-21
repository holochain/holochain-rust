extern crate hc_dna;
extern crate snowflake;

use hc_dna::Dna;

pub mod ribosome;

//use self::ribosome::*;
use state;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NucleusState {
    dna: Option<Dna>,
    initialized: bool,
    ribosome_calls: HashMap<FunctionCall, Option<String>>,
}

impl NucleusState {
    pub fn new() -> Self {
        NucleusState {
            dna: None,
            initialized: false,
            ribosome_calls: HashMap::new(),
        }
    }

    pub fn dna(&self) -> Option<Dna> {
        self.dna.clone()
    }
    pub fn initialized(&self) -> bool {
        self.initialized
    }
    pub fn ribosome_call_result(&self, function_call: &FunctionCall) -> Option<String> {
        match self.ribosome_calls.get(function_call) {
            None => None,
            Some(value) => value.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionCall {
    id: snowflake::ProcessUniqueId,
    pub zome: String,
    pub capability: String,
    pub function: String,
    pub parameters: String,
}

impl FunctionCall {
    pub fn new(zome: String, capability: String, function: String, parameters: String) -> Self {
        FunctionCall {
            id: snowflake::ProcessUniqueId::new(),
            zome: zome,
            capability: capability,
            function: function,
            parameters: parameters,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionResult {
    call: FunctionCall,
    result: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    InitApplication(Dna),
    ExecuteZomeFunction(FunctionCall),
    ZomeFunctionResult(FunctionResult),
}

pub fn reduce(
    old_state: Arc<NucleusState>,
    action: &state::Action,
    action_channel: &Sender<state::ActionWrapper>,
) -> Arc<NucleusState> {
    match *action {
        state::Action::Nucleus(ref nucleus_action) => {
            let mut new_state: NucleusState = (*old_state).clone();
            match *nucleus_action {
                Action::InitApplication(ref dna) => {
                    if !new_state.initialized {
                        new_state.dna = Some(dna.clone());
                        new_state.initialized = true;
                    }
                }

                Action::ExecuteZomeFunction(ref fc) => {
                    let function_call = fc.clone();
                    if let Some(ref dna) = new_state.dna {
                        if let Some(ref wasm) =
                            dna.get_wasm_for_capability(&fc.zome, &fc.capability)
                        {
                            new_state.ribosome_calls.insert(fc.clone(), None);

                            let action_channel = action_channel.clone();
                            let code = wasm.code.clone();
                            thread::spawn(move || {
                                match ribosome::call(code, &function_call.function.clone()) {
                                    Ok(runtime) => {
                                        let mut result = FunctionResult {
                                            call: function_call,
                                            result: runtime.result.to_string(),
                                        };

                                        action_channel
                                            .send(state::ActionWrapper::new(
                                                state::Action::Nucleus(Action::ZomeFunctionResult(
                                                    result,
                                                )),
                                            ))
                                            .expect("action channel to be open in reducer");
                                    }

                                    Err(ref error) => {
                                        println!("Error calling ribosome: {}", error);
                                        panic!("Error calling ribosome: {}\nWe have to handle that by storing any error in the state...", error);
                                    }
                                }
                            });
                        }
                    }
                }

                Action::ZomeFunctionResult(ref result) => {
                    new_state
                        .ribosome_calls
                        .insert(result.call.clone(), Some(result.result.clone()));
                }
            }
            Arc::new(new_state)
        }
        _ => old_state,
    }
}

#[cfg(test)]
mod tests {
    use super::super::nucleus::Action::*;
    use super::super::state::Action::*;
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn can_instantiate_nucleus_state() {
        let state = NucleusState::new();
        assert_eq!(state.dna, None);
        assert_eq!(state.initialized, false);
    }

    #[test]
    fn can_reduce_initialize_action() {
        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna));
        let state = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = channel::<state::ActionWrapper>();
        let reduced_state = reduce(state.clone(), &action, &sender.clone());
        assert!(reduced_state.initialized, true);

        // on second reduction it still works.
        let second_reduced_state = reduce(reduced_state.clone(), &action, &sender.clone());
        assert_eq!(second_reduced_state, reduced_state);
    }

    #[test]
    fn can_reduce_execfn_action() {
        let call = FunctionCall::new(
            "myZome".to_string(),
            "public".to_string(),
            "bogusfn".to_string(),
            "".to_string(),
        );

        let action = Nucleus(ExecuteZomeFunction(call));
        let state = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = channel::<state::ActionWrapper>();
        let reduced_state = reduce(state.clone(), &action, &sender);
        assert_eq!(state, reduced_state);
    }
}
