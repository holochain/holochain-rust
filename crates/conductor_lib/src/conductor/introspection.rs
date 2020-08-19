use crate::conductor::Conductor;
use holochain_core_types::{
    dna::fn_declarations::{FnDeclaration, TraitFns},
    error::HolochainError,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZomePath {
    pub instance_id: String,
    pub zome_name: String,
}

pub trait ConductorIntrospection {
    fn get_zomes_by_trait(
        &mut self,
        trait_name: String,
        trait_functions: Vec<FnDeclaration>,
    ) -> Result<Vec<ZomePath>, HolochainError>;
}

impl ConductorIntrospection for Conductor {
    fn get_zomes_by_trait(
        &mut self,
        trait_name: String,
        trait_functions: Vec<FnDeclaration>,
    ) -> Result<Vec<ZomePath>, HolochainError> {
        let mut result = Vec::new();
        for (instance_id, instance_lock) in self.instances.iter() {
            if let Ok(dna) = instance_lock.read()?.dna() {
                // Instance is initialized and has DNA
                for (zome_name, zome) in dna.zomes.iter() {
                    if let Some(TraitFns { functions }) = zome.traits.get(&trait_name) {
                        // DNA implements a trait with same name.
                        // Still need to check all functions signatures...
                        let mut is_good = true;
                        for trait_fn_decl in trait_functions.iter() {
                            // Is the function name declared (by the zome) as part of that trait
                            // and does the whole declaration (complete signature) of the function
                            // as found in the some match the declaration in the given trait?
                            if !functions.contains(&trait_fn_decl.name)
                                || !zome.fn_declarations.contains(&trait_fn_decl)
                            {
                                // If not we can exit this loop since it takes only one missing
                                // function to not have the trait implemented.
                                is_good = false;
                                break;
                            }
                        }

                        // If we didn't break out of above loop, we could fine all functions
                        // and we're good - this zome implements the trait.
                        if is_good {
                            let instance_id = instance_id.clone();
                            let zome_name = zome_name.clone();
                            result.push(ZomePath {
                                instance_id,
                                zome_name,
                            })
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}
