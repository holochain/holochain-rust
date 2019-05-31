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

        quote! {
            #[no_mangle]
            #[allow(unused_variables)]
            pub extern "C" fn zome_setup(zd: &mut hdk::meta::ZomeDefinition) {
                #(
                    zd.define(#entry_fn_idents ());
                )*
            }
        }
    }
}
