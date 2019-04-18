#![recursion_limit = "256"]
#![feature(try_from, proc_macro_diagnostic)]

extern crate hdk;
extern crate proc_macro;
extern crate proc_macro2;

use crate::into_zome::IntoZome;
use proc_macro2::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn;

mod into_zome;
mod to_tokens;
mod types;

use crate::types::{FnDeclaration, ZomeCodeDef, ZomeFunctionCode};

// use this to convert from the tagged #[zome] module into a definition struct
impl TryFrom<TokenStream> for ZomeCodeDef {
    type Error = syn::Error;

    fn try_from(input: TokenStream) -> Result<Self, Self::Error> {
        let module: syn::ItemMod = syn::parse(input.into())?;
        Ok(module.extract_zome())
    }
}

// use this to convert back to a token stream usable by the compiler
impl ZomeCodeDef {
    fn to_wasm_friendly(&self) -> TokenStream {
        let genesis = &self.genesis;
        let (_zome_fn_defs, _): (Vec<FnDeclaration>, Vec<ZomeFunctionCode>) = self
            .zome_fns
            .clone()
            .into_iter()
            .map(|e| (e.declaration, e.code))
            .unzip();
        let zome_fns = self.zome_fns.clone();

        let entry_def_fns = self.entry_def_fns.clone();
        let entry_fn_idents = self
            .entry_def_fns
            .iter()
            .map(|func| func.ident.clone())
            .clone();
        let extra = &self.extra;

        let (receive_blocks, receive_params) = match &self.receive_callback {
            None => (Vec::new(), Vec::new()),
            Some(callback) => (vec![callback.code.clone()], vec![callback.param.clone()]),
        };

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

        let gen = quote! {

            #(#extra)*

            #(#entry_def_fns )*

            #[no_mangle]
            #[allow(unused_variables)]
            pub extern "C" fn zome_setup(zd: &mut hdk::meta::ZomeDefinition) {
                #(
                    zd.define(#entry_fn_idents ());
                )*
            }

            #[no_mangle]
            pub extern "C" fn genesis(encoded_allocation_of_input: hdk::holochain_core_types::error::RibosomeEncodingBits) -> hdk::holochain_core_types::error::RibosomeEncodingBits {
                let maybe_allocation = hdk::holochain_wasm_utils::memory::allocation::WasmAllocation::try_from_ribosome_encoding(encoded_allocation_of_input);
                let allocation = match maybe_allocation {
                    Ok(allocation) => allocation,
                    Err(allocation_error) => return hdk::holochain_core_types::error::RibosomeEncodedValue::from(allocation_error).into(),
                };
                let init = hdk::global_fns::init_global_memory(allocation);
                if init.is_err() {
                    return hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                        init
                    ).into();
                }

                fn execute() -> Result<(), String> {
                    #genesis
                }

                match execute() {
                    Ok(_) => hdk::holochain_core_types::error::RibosomeEncodedValue::Success.into(),
                    Err(e) => hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                        hdk::global_fns::write_json(
                            hdk::holochain_wasm_utils::holochain_core_types::json::RawString::from(e)
                        )
                    ).into(),
                }
            }

            #[no_mangle]
            #[allow(unused_imports)]
            pub fn __list_traits() -> hdk::holochain_core_types::dna::zome::ZomeTraits {
                // use std::collections::BTreeMap;
                // BTreeMap::new()

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


            #(
                #[no_mangle]
                pub extern "C" fn receive(encoded_allocation_of_input: hdk::holochain_core_types::error::RibosomeEncodingBits) -> hdk::holochain_core_types::error::RibosomeEncodingBits {
                    let maybe_allocation = hdk::holochain_wasm_utils::memory::allocation::WasmAllocation::try_from_ribosome_encoding(encoded_allocation_of_input);
                    let allocation = match maybe_allocation {
                        Ok(allocation) => allocation,
                        Err(allocation_error) => return hdk::holochain_core_types::error::RibosomeEncodedValue::from(allocation_error).into(),
                    };
                    let init = hdk::global_fns::init_global_memory(allocation);
                    if init.is_err() {
                        return hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                            init
                        ).into();
                    }

                    // Deserialize input
                    let input = load_string!(encoded_allocation_of_input);

                    fn execute(payload: String) -> String {
                        let #receive_params = payload;
                        #receive_blocks
                    }

                    hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                        hdk::global_fns::write_json(
                            JsonString::from_json(&execute(input))
                        )
                    ).into()
                }
            )*

            #[no_mangle]
            #[allow(unused_imports)]
            pub fn __list_functions() -> hdk::holochain_core_types::dna::zome::ZomeFnDeclarations {
                use hdk::holochain_core_types::dna::fn_declarations::{FnParameter, FnDeclaration};
                vec![#(#_zome_fn_defs,)*]
            }

            #[no_mangle]
            pub extern "C" fn __install_panic_handler() -> () {
                use hdk::{api::debug, holochain_core_types::json::RawString};
                use std::panic;
                panic::set_hook(Box::new(move |info| {
                    let _ = debug(RawString::from(
                        info.payload().downcast_ref::<String>().unwrap().clone(),
                    ));

                    let _ = if let Some(location) = info.location() {
                        debug(RawString::from(format!(
                            "panic occurred in file '{}' at line {}",
                            location.file(),
                            location.line()
                        )))
                    } else {
                        debug(RawString::from(format!(
                            "panic occurred but can't get location information..."
                        )))
                    };
                }));
            }

            #(#zome_fns )*

        };

        gen.into()
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
