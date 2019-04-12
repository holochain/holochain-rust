use crate::dna::fn_declarations::{FnDeclaration, FnParameter};
use proc_macro2::{Ident, Span};
/**
 * Implementation of the quote::ToTokens trait for various structs used in the proc-macro hdk
 */
use quote::{ToTokens, __rt::TokenStream};
use syn;

impl ToTokens for FnParameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let input_param_name = &self.name;
        let input_param_type = &self.parameter_type;
        tokens.extend(quote! {
            FnParameter::new(#input_param_name, #input_param_type)
        })
    }
}

#[allow(dead_code)]
impl From<FnParameter> for syn::Field {
    fn from(param: FnParameter) -> Self {
        let ident = Ident::new(&param.name, Span::call_site());
        let ty: syn::Type = syn::parse_str(&param.parameter_type).unwrap();
        syn::Field {
            attrs: Vec::new(),
            ident: Some(ident),
            ty,
            vis: syn::Visibility::Public(syn::VisPublic {
                pub_token: Token![pub](Span::call_site()),
            }),
            colon_token: Some(Token![:](proc_macro2::Span::call_site())),
        }
    }
}

impl ToTokens for FnDeclaration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zome_function_name = &self.name;
        let input_param_names = &self.inputs;
        let output_param_names = &self.outputs;

        tokens.extend(quote! {
            FnDeclaration {
                name: #zome_function_name.to_string(),
                inputs: vec![#(#input_param_names,)*],
                outputs: vec![#(#output_param_names,)*],
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_fn_params() {
        let params = FnParameter::new("input", "String");
        assert_eq!(
            params.into_token_stream().to_string(),
            r#"FnParameter :: new ( "input" , "String" )"#
        )
    }

    #[test]
    fn test_to_struct_tokens() {
        let params = FnParameter::new("input", "String");
        let field: syn::Field = params.into();
        assert_eq!(quote! {#field}.to_string(), r#"pub input : String"#)
    }

    #[test]
    fn test_tokenize_fn_def() {
        let inputs = vec![FnParameter::new("input", "String")];
        let outputs = vec![FnParameter::new("output", "String")];

        let func_dec = FnDeclaration {
            name: "test_func".to_string(),
            inputs,
            outputs,
        };

        assert_eq!(
        	func_dec.into_token_stream().to_string(),
        	r#"FnDeclaration { name : "test_func" . to_string ( ) , inputs : vec ! [ FnParameter :: new ( "input" , "String" ) , ] , outputs : vec ! [ FnParameter :: new ( "output" , "String" ) , ] , }"#
        )
    }
}
