#![recursion_limit="256"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;


/**
 * @brief      Macro to be used on a Rust module. The contents of the module is processed and exported as a zome
 */
#[proc_macro_attribute]
pub fn zome(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let _ast: syn::ItemMod = syn::parse(input).unwrap();

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
                Ok(())
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
            }));        }
    };

    gen.into()
}