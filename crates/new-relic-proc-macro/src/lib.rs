extern crate proc_macro;
extern crate quote;
extern crate syn;
extern crate newrelic;
use proc_macro::TokenStream;
use quote::quote;
use std::env;

//this will only transform the function if item is available
//this will be moved to tracing crate soon
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_clone = item.clone();
    env::var("NEW_RELIC_LICENSE_KEY").map(|license_key|
    {
        let input = syn::parse_macro_input!(item_clone as syn::ItemFn);
        let app_name = attr.clone().into_iter().nth(0).map(|token|token.to_string()).unwrap_or("UNDEFINED".to_string());
        let transaction_type = attr.clone().into_iter().nth(2).map(|token|token.to_string()).unwrap_or("no_license_key".to_string());
        let category = attr.clone().into_iter().nth(3).map(|token|token.to_string()).unwrap_or("DEFAULT".to_string());
        
        //function declaration for redifining because toToken for fndecl is not available
        let function_name = input.ident.to_string();
        let visibility = input.vis;
        let fn_name = input.ident;
        let arguments = input.decl.inputs;
        let block = input.block;
        let asyncness = input.asyncness;
        let output = input.decl.output;
        let generic = input.decl.generics;
        let where_for_generics = generic.where_clause.clone();

        //structure of func created will replce old function but have new relic recording capabilities
        let result = quote!{
            #visibility #asyncness fn #fn_name#generic(#arguments) #output #where_for_generics
            {
                newrelic::App::new(#app_name, #license_key)
                .map(|live_app|{
                    live_app.non_web_transaction(#transaction_type)
                            .map(|transaction|{
                                transaction.custom_segment(#function_name,#category,|_|#block)
                            }).unwrap_or_else(|_|#block)
                }).unwrap_or_else(|_|#block)
            }
        };
        result.into()
    }).unwrap_or(item)
    
}

