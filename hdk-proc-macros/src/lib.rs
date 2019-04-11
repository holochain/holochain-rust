#![recursion_limit="256"]
#![feature(try_from, proc_macro_diagnostic)]


extern crate proc_macro;
extern crate hdk;

use std::convert::{TryFrom};
use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

static GENESIS_ATTRIBUTE: &str = "genesis";
static ZOME_FN_ATTRIBUTE: &str = "zome_fn";


use hdk::holochain_core_types::{
    dna::{
        fn_declarations::{FnDeclaration},
    }
};


type GenesisCallback = syn::Block;
type ZomeFunctionCode = syn::Block;
type ZomeFunctions = Vec<(FnDeclaration, ZomeFunctionCode)>;

// type ReceiveCallbacks = Vec<syn::Block>;

struct ZomeCodeDef {
    // zome: Zome,
    genesis: GenesisCallback,
    zome_fns: ZomeFunctions
    // receive: ReceiveCallbacks,
}

fn is_tagged_with(attrs: &Vec<syn::Attribute>, tag: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.path.is_ident(tag)
    })
}

fn zome_fn_dec_from_syn(func: &syn::ItemFn) -> FnDeclaration {
    // let inputs = func.decl.inputs;

    FnDeclaration {
        name: func.ident.clone().to_string(),
        inputs: Vec::new(),
        outputs: Vec::new(),
    }
}

fn extract_genesis(module: &syn::ItemMod) -> GenesisCallback {
    // find all the functions tagged as the genesis callback
    let geneses: Vec<Box<syn::Block>> = module.clone().content.unwrap().1.into_iter()
    .fold(Vec::new(), |mut acc, item| {
        if let syn::Item::Fn(func) = item {
            if is_tagged_with(&func.attrs, GENESIS_ATTRIBUTE) {
                acc.push(func.block)
            }
        } 
        acc
    });
    // only a single function can be tagged in a valid some so error if there is more than one
    // if there is None then use the sensible default of Ok(())
    match geneses.len() {
        0 => {
            module.ident.span().unstable()
            .error("No genesis function defined! A zome definition requires a callback tagged with #[genesis]")
            .emit();
            panic!()
        },
        1 => *geneses[0].clone(),
        _ => {
            module.ident.span().unstable()
            .error("Multiple functions tagged as genesis callback! Only one is permitted per zome definition.")
            .emit();
            panic!()
        }
    }
}

fn extract_zome_fns(module: &syn::ItemMod) -> ZomeFunctions {
    // find all the functions tagged as the zome_fn
    module.clone().content.unwrap().1.into_iter()
    .fold(Vec::new(), |mut acc, item| {
        if let syn::Item::Fn(func) = item {
            if is_tagged_with(&func.attrs, ZOME_FN_ATTRIBUTE) {

                let fn_def = zome_fn_dec_from_syn(&func);

                acc.push((fn_def, *func.block))
            }
        } 
        acc
    })
}

// use this to convert from the tagged #[zome] module into a definition struct
impl TryFrom<TokenStream> for ZomeCodeDef {
    type Error = syn::Error;

    fn try_from(input: TokenStream) -> Result<Self, Self::Error> {
        let module: syn::ItemMod = syn::parse(input)?;

        Ok(
            ZomeCodeDef {
                genesis: extract_genesis(&module),
                zome_fns: extract_zome_fns(&module)
            }
        )
    }
}

// use this to convert back to a token stream usable by the compiler
impl ZomeCodeDef {

    fn to_wasm_friendly(&self) -> TokenStream {

        let genesis = &self.genesis;
        let (_zome_fn_defs, _zome_fn_code): (Vec<FnDeclaration>, Vec<ZomeFunctionCode>) = self.zome_fns.clone().into_iter().unzip();

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
