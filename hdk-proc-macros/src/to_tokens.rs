use crate::zome_code_def::{
    FnDeclaration, FnParameter, ZomeCodeDef, ZomeFunction, ZomeFunctionCode,
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

impl ToTokens for ZomeFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zome_function_name = Ident::new(&self.declaration.name, Span::call_site());

        let input_params = self
            .declaration
            .inputs
            .clone()
            .into_iter()
            .map(|param| syn::Field::from(param));

        let input_param_names = self
            .declaration
            .inputs
            .clone()
            .into_iter()
            .map(|param| param.ident.clone());

        let output_param_type = &self.declaration.output;
        let function_body = &self.code;

        tokens.extend(quote!{
            #[no_mangle]
            pub extern "C" fn #zome_function_name(encoded_allocation_of_input: hdk::holochain_core_types::error::RibosomeEncodingBits) -> hdk::holochain_core_types::error::RibosomeEncodingBits {
                use hdk::{
                    holochain_core_types::{
                        json::JsonString,
                        error::HolochainError
                    },
                };

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

                // Macro'd InputStruct
                #[derive(Deserialize, Serialize, Debug, hdk::holochain_core_types_derive::DefaultJson)]
                struct InputStruct {
                    #(#input_params),*
                }

                // Deserialize input
                let input: InputStruct = hdk::load_json!(encoded_allocation_of_input);

                // Macro'd function body
                fn execute (params: InputStruct) #output_param_type {
                    let InputStruct { #(#input_param_names),* } = params;
                    #function_body
                }

                hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                    hdk::global_fns::write_json(execute(input))
                ).into()
            }
        })
    }
}

impl ToTokens for FnParameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let input_param_name = &self.ident;

        let input_param_type = &self.ty;

        tokens.extend(quote! {
            FnParameter::new(stringify!(#input_param_name), stringify!(#input_param_type))
        })
    }
}

impl ToTokens for FnDeclaration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zome_function_name = &self.name;
        let input_params = &self.inputs;
        let output_params = match &self.output {
            syn::ReturnType::Default => Vec::new(),
            syn::ReturnType::Type(_, ty) => {
                vec![quote!(FnParameter::new("result", stringify!(#ty)))]
            }
        };

        tokens.extend(quote! {
            FnDeclaration {
                name: #zome_function_name.to_string(),
                inputs: vec![#(#input_params,)*],
                outputs: vec![#(#output_params,)*],
            }
        })
    }
}

pub fn panic_handler() -> TokenStream {
    quote! {
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
    }
}

impl ZomeCodeDef {
    pub fn genesis(&self) -> TokenStream {
        let genesis = &self.genesis;

        quote! {
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
        }
    }

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

    pub fn receive_callback(&self) -> TokenStream {
        let (receive_blocks, receive_params) = match &self.receive_callback {
            None => (Vec::new(), Vec::new()),
            Some(callback) => (vec![callback.code.clone()], vec![callback.param.clone()]),
        };

        quote! {
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
        }
    }

    pub fn to_wasm_friendly(&self) -> TokenStream {
        let genesis = self.genesis();
        let zome_setup = self.zome_setup();
        let list_traits = self.list_traits();
        let list_functions = self.list_functions();
        let zome_fns = self.zome_fns.clone();
        let entry_def_fns = self.entry_def_fns.clone();
        let extra = &self.extra;
        let receive_callback = self.receive_callback();
        let panic_handler = panic_handler();

        let gen = quote! {

            #(#extra)*

            #(#entry_def_fns )*

            #genesis

            #zome_setup

            #list_traits

            #list_functions

            #receive_callback

            #panic_handler

            #(#zome_fns )*

        };

        gen.into()
    }
}
