use crate::zome_code_def::ZomeCodeDef;
use proc_macro2::TokenStream;
use quote::quote;

impl ZomeCodeDef {
    pub fn list_traits(&self) -> TokenStream {
        let traits = self.traits.iter().map(|(tr8, trait_funcs)| {
            let funcs = trait_funcs.functions.clone();

            quote! {
                {
                    let mut traitfns = TraitFns::new();
                    traitfns.functions = vec![
                        #(
                            #funcs.into()
                        ),*
                    ];

                    traitfns_map.insert(#tr8.into(), traitfns);
                }
            }
        });

        quote! {
            #[no_mangle]
            #[allow(unused_imports)]
            pub fn __list_traits() -> hdk::holochain_core_types::dna::zome::ZomeTraits {
                use hdk::holochain_core_types::dna::{
                    fn_declarations::{FnParameter, FnDeclaration, TraitFns},
                };
                use std::collections::BTreeMap;

                let return_value: hdk::holochain_core_types::dna::zome::ZomeTraits = {
                    let mut traitfns_map = BTreeMap::new();

                    #(
                        #traits
                    )*

                    traitfns_map
                };

                return_value
            }
        }
    }
}
