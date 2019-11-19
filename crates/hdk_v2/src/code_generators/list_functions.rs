use crate::zome_code_def::{FnDeclaration, ZomeCodeDef, ZomeFunctionCode};
use proc_macro2::TokenStream;
use quote::quote;

impl ZomeCodeDef {
    pub fn list_functions(&self) -> TokenStream {
        let (_zome_fn_defs, _): (Vec<FnDeclaration>, Vec<ZomeFunctionCode>) = self
            .zome_fns
            .clone()
            .into_iter()
            .map(|e| (e.declaration, e.code))
            .unzip();

        quote! {
            #[no_mangle]
            #[allow(unused_imports)]
            pub fn __list_functions() -> hdk::holochain_core_types::dna::zome::ZomeFnDeclarations {
                use hdk::holochain_core_types::dna::fn_declarations::{FnParameter, FnDeclaration};
                vec![#(#_zome_fn_defs,)*]
            }
        }
    }
}
