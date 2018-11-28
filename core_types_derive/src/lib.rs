#![recursion_limit = "128"]
#![cfg_attr(tarpaulin, skip)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

fn impl_default_json_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {

        impl<'a> From<&'a #name> for JsonString {
            fn from(v: &#name) -> JsonString {
                match ::serde_json::to_string(v) {
                    Ok(s) => Ok(JsonString::from(s)),
                    Err(e) => Err(HolochainError::SerializationError(e.to_string())),
                }.expect(&format!("could not Jsonify {}: {:?}", stringify!(#name), v))
            }
        }

        impl From<#name> for JsonString {
            fn from(v: #name) -> JsonString {
                JsonString::from(&v)
            }
        }

        impl<'a> ::std::convert::TryFrom<&'a JsonString> for #name {
            type Error = HolochainError;
            fn try_from(json_string: &JsonString) -> Result<Self, Self::Error> {
                match ::serde_json::from_str(&String::from(json_string)) {
                    Ok(d) => Ok(d),
                    Err(e) => Err(HolochainError::SerializationError(e.to_string())),
                }
            }
        }

        impl ::std::convert::TryFrom<JsonString> for #name {
            type Error = HolochainError;
            fn try_from(json_string: JsonString) -> Result<Self, Self::Error> {
                #name::try_from(&json_string)
            }
        }

    };
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
