#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

fn impl_default_json_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl From<#name> for JsonString {
            fn from(v: #name) -> JsonString {
                // ::holochain_core_types::json::default_from_json(v)
                match ::serde_json::to_string(&v) {
                    Ok(s) => Ok(JsonString::from(s)),
                    Err(e) => Err(HolochainError::SerializationError(e.to_string())),
                }.unwrap()
            }
        }

        // impl TryFrom<JsonString> for #name {
        //     type Error = ::holochain_core_types::error::HolochainError;
        //     fn try_from(json_string: ::holochain_core_types::json::JsonString) -> Result<Self, Self::Error> {
        //         match ::serde_json::from_str(&String::from(&json_string)) {
        //             Ok(d) => Ok(d),
        //             Err(e) => Err(::holochain_core_types::error::HolochainError::SerializationError(e.to_string())),
        //         }
        //     }
        // }

    };
    // panic!(gen.to_string());
    gen.into()
}

#[proc_macro_derive(DefaultJson)]
pub fn default_json_derive(input: TokenStream) -> TokenStream {
    // Construct a represntation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_default_json_macro(&ast)
}
