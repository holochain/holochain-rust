#![recursion_limit="256"]
#![feature(try_from, proc_macro_diagnostic)]


extern crate proc_macro;
extern crate hdk;

use std::convert::{TryFrom};
use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

static GENESIS_ATTRIBUTE: &str = "genesis";

// use hdk::holochain_core_types::{
//     dna::{
//         zome::Zome,
//         fn_declarations::{FnDeclaration, FnParameter, TraitFns},
//     }
// };


type GenesisCallback = syn::Block;
// type ReceiveCallbacks = Vec<syn::Block>;

struct ZomeCodeDef {
    // zome: Zome,
    genesis: GenesisCallback,
    // receive: ReceiveCallbacks,
}

fn is_tagged_genesis(attrs: Vec<syn::Attribute>) -> bool {
    attrs.iter().any(|attr| {
        attr.path.is_ident(GENESIS_ATTRIBUTE)
    })
}

// use this to convert from the tagged #[zome] module into a definition struct
impl TryFrom<TokenStream> for ZomeCodeDef {
    type Error = syn::Error;

    fn try_from(input: TokenStream) -> Result<Self, Self::Error> {
        let module: syn::ItemMod = syn::parse(input)?;

        // find all the functions tagged as the genesis callback
        let geneses: Vec<Box<syn::Block>> = module.content.unwrap().1.into_iter()
        .fold(Vec::new(), |mut acc, item| {
            if let syn::Item::Fn(func) = item {
                if is_tagged_genesis(func.attrs) {
                    acc.push(func.block)
                }
            } 
            acc
        });
        // only a single function can be tagged in a valid some so error if there is more than one
        // if there is None then use the sensible default of Ok(())
        let genesis = match geneses.len() {
            0 => {
                module.ident.span().unstable()
                .error("No genesis function defined! A zome definition requires a callback tagged with #[genesis]")
                .emit();
                panic!()
            },
            1 => &geneses[0],
            _ => {
                module.ident.span().unstable()
                .error("Multiple functions tagged as genesis callback! Only one is permitted per zome definition.")
                .emit();
                panic!()
            }
        };


        Ok(
            ZomeCodeDef{
                genesis: *genesis.clone()
            }
        )
    }
}

// use this to convert back to a token stream usable by the compiler
impl ZomeCodeDef {

    fn to_wasm_friendly(&self) -> TokenStream {

        let genesis = &self.genesis;

        let gen = quote!{

            #[no_mangle]
            #[allow(unused_variables)]
            pub extern "C" fn zome_setup(zd: &mut hdk::meta::ZomeDefinition) {

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
                use std::collections::BTreeMap;
                BTreeMap::new()
            }

            #[no_mangle]
            #[allow(unused_imports)]
            pub fn __list_functions() -> hdk::holochain_core_types::dna::zome::ZomeFnDeclarations {
                Vec::new()
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
        };

        gen.into()
    }   
}

/**
 * @brief      Macro to be used on a Rust module. The contents of the module is processed and exported as a zome
 */
#[proc_macro_attribute]
pub fn zome(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    ZomeCodeDef::try_from(input)
        .unwrap()
        .to_wasm_friendly()
}
