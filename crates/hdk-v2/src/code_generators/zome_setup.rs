use crate::zome_code_def::ZomeCodeDef;
use proc_macro2::TokenStream;
use quote::quote;

impl ZomeCodeDef {
    pub fn zome_setup(&self) -> TokenStream {
        let entry_fn_idents = self
            .entry_def_fns
            .iter()
            .map(|func| func.ident.clone())
            .clone();

        let agent_validation_param = &self.validate_agent.validation_data_param;
        let agent_validation_expr = &self.validate_agent.code;

        quote! {
            #[no_mangle]
            #[allow(unused_variables)]
            pub extern "C" fn zome_setup(zd: &mut hdk::meta::ZomeDefinition) {
                #(
                    zd.define(#entry_fn_idents ());
                )*
                let validator = Box::new(|validation_data: hdk::holochain_wasm_utils::holochain_core_types::validation::EntryValidationData<hdk::holochain_core_types::agent::AgentId>| {
                    let #agent_validation_param = validation_data;
                    #agent_validation_expr
                });
                zd.define_agent_validator(validator);
            }
        }
    }
}
