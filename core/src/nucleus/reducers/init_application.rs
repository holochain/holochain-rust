use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::state::{NucleusState, NucleusStatus},
};
use std::sync::Arc;
use crate::nucleus::ribosome::wasmi_factory::wasmi_factory;
use holochain_core_types::error::HolochainError;
use holochain_core_types::dna::zome::Zome;
use crate::nucleus::state::ModuleRefMutex;

/// Reduce InitializeChain Action
/// Switch status to failed if an initialization is tried for an
/// already initialized, or initializing instance.
#[allow(unknown_lints)]
#[allow(needless_pass_by_value)]
pub fn reduce_initialize_chain(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    match state.status() {
        NucleusStatus::Initializing => {
            state.status =
                NucleusStatus::InitializationFailed("Nucleus already initializing".to_string())
        }
        NucleusStatus::Initialized(_) => {
            state.status =
                NucleusStatus::InitializationFailed("Nucleus already initialized".to_string())
        }
        NucleusStatus::New | NucleusStatus::InitializationFailed(_) => {
            let ia_action = action_wrapper.action();
            let dna = unwrap_to!(ia_action => Action::InitializeChain);
            // Update status
            state.status = NucleusStatus::Initializing;
            // Set DNA
            state.dna = Some(dna.clone());
            // Create Ribosomes
            for (zome_name, zome) in dna.zomes.iter() {
                match create_ribosomes_for_zome(zome) {
                    Ok(pool) => {
                        state.ribosomes.insert(zome_name.clone(), pool);
                    },
                    Err(err) => {
                        context.log(format!("err/nucleus/initialize: Could not create ribosome: {:?}", err));
                    }
                };
            }
        }
    }
}

/// Creates a pool of 8 WASM module instances all with the same code from the given zome
fn create_ribosomes_for_zome(zome: &Zome) -> Result<Vec<ModuleRefMutex>, HolochainError> {
    let mut pool = Vec::new();
    for _i in 1..8 {
        let ribosome = wasmi_factory(zome.code.code.clone())?;
        pool.push(ModuleRefMutex::new(ribosome));
    };
    Ok(pool)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        action::ActionWrapper,
        instance::{tests::test_context_with_channels, Observer},
        nucleus::{
            reduce,
            state::{NucleusState, NucleusStatus},
        },
    };
    use holochain_core_types::dna::Dna;
    use std::sync::{mpsc::sync_channel, Arc};

    #[test]
    /// smoke test the init of a nucleus reduction
    fn can_reduce_initialize_action() {
        let dna = Dna::new();
        let action_wrapper = ActionWrapper::new(Action::InitializeChain(dna.clone()));
        let nucleus = Arc::new(NucleusState::new()); // initialize to bogus value
        let (sender, _receiver) = sync_channel::<ActionWrapper>(10);
        let (tx_observer, _observer) = sync_channel::<Observer>(10);
        let context = test_context_with_channels("jimmy", &sender, &tx_observer, None);

        // Reduce Init action
        let reduced_nucleus = reduce(context.clone(), nucleus.clone(), &action_wrapper);

        assert_eq!(reduced_nucleus.has_initialized(), false);
        assert_eq!(reduced_nucleus.has_initialization_failed(), false);
        assert_eq!(reduced_nucleus.status(), NucleusStatus::Initializing);
        assert!(reduced_nucleus.dna().is_some());
        assert_eq!(reduced_nucleus.dna().unwrap(), dna);
    }
}
