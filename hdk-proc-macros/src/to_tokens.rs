use crate::types::{FnDeclaration, FnParameter, ZomeFunction};

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
