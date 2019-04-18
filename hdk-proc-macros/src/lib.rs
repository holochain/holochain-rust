#![recursion_limit = "256"]
#![feature(try_from, proc_macro_diagnostic)]

extern crate hdk;
extern crate proc_macro;
extern crate proc_macro2;

use crate::into_zome::IntoZome;
use proc_macro2::TokenStream;
use std::convert::TryFrom;
use syn;

mod into_zome;
mod to_tokens;
mod types;

use crate::types::ZomeCodeDef;

// use this to convert from the tagged #[zome] module into a definition struct
impl TryFrom<TokenStream> for ZomeCodeDef {
    type Error = syn::Error;

    fn try_from(input: TokenStream) -> Result<Self, Self::Error> {
        let module: syn::ItemMod = syn::parse(input.into())?;
        Ok(module.extract_zome())
    }
}

/**
 * @brief      Macro to be used on a Rust module. The contents of the module is processed and exported as a zome
 */
#[proc_macro_attribute]
pub fn zome(
    _metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input_stream: TokenStream = input.into();
    ZomeCodeDef::try_from(input_stream)
        .unwrap()
        .to_wasm_friendly()
        .into()
}
