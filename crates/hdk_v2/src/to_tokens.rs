use crate::zome_code_def::{FnDeclaration, FnParameter, ZomeCodeDef, ZomeFunction};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::code_generators::panic_handler;

impl ToTokens for ZomeFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zome_function_name = Ident::new(&self.declaration.name, Span::call_site());

        let input_params = self
            .declaration
            .inputs
            .clone()
            .into_iter()
            .map(syn::Field::from);

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
                    holochain_json_api::{
                        json::JsonString,
                        error::JsonError
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
                #[derive(Deserialize, Serialize, Debug, hdk::holochain_json_derive::DefaultJson)]
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

impl ToTokens for ZomeCodeDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let init = self.init();
        let zome_setup = self.zome_setup();
        let list_traits = self.list_traits();
        let list_functions = self.list_functions();
        let zome_fns = self.zome_fns.clone();
        let entry_def_fns = self.entry_def_fns.clone();
        let extra = &self.extra;
        let receive_callback = self.receive_callback();
        let panic_handler = panic_handler();

        tokens.extend(quote! {

            #(#extra)*

            #(#entry_def_fns )*

            #init

            #zome_setup

            #list_traits

            #list_functions

            #receive_callback

            #panic_handler

            #(#zome_fns )*

        });
    }
}
