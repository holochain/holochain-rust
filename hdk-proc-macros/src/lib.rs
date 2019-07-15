#![recursion_limit = "256"]
#![feature(proc_macro_diagnostic)]

extern crate hdk;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::TokenStream;
use quote::ToTokens;
use std::convert::TryFrom;

mod code_generators;
mod into_zome;
mod to_tokens;
mod zome_code_def;

use crate::zome_code_def::ZomeCodeDef;

/**
 * @brief Defines the #[zome] macro to be used on a Rust module.
 * The contents of the module is processed into a ZomeCodeDef and then re-exported as wasm friendly code
 */
#[proc_macro_attribute]
pub fn zome(
    _metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input_stream: TokenStream = input.into();
    ZomeCodeDef::try_from(input_stream)
        .unwrap()
        .into_token_stream()
        .into()
}
