//! Attribute proc-macro are defined here for easy to use.
//!
//! It's intended to be used with the `holochain_metrics` crate like this:
//! ```
//! use holochain_metrics::prelude::*;
//!
//! #[latency]
//! pub fn invoke_attr_macro() {
//!     println!("Attribute Macro Invoked.");
//!     let _x = 3 + 4;
//! }

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Attribute, FnArg, Ident, ItemFn, PatType, ReturnType};

#[proc_macro_attribute]
pub fn latency(args: TokenStream, input_function: TokenStream) -> TokenStream {
    // Let's make sure there is no argument passed to the latency attribute macro
    assert!(args.is_empty());

    let input_function_cloned = input_function.clone();
    let mut function = parse_macro_input!(input_function_cloned as ItemFn);

    let metric_name = &format!("{}.latency", &function.sig.ident.to_string());

    // Boiler plate ...start
    let mut move_self = None;
    let mut arg_pat = Vec::new();
    let mut arg_val = Vec::new();
    for (i, input) in function.sig.inputs.iter_mut().enumerate() {
        let numbered = Ident::new(&format!("__arg{}", i), Span::call_site());

        match input {
            FnArg::Typed(PatType { pat, .. }) => {
                arg_pat.push(quote!(#pat));
                arg_val.push(quote!(#numbered));
                *pat = parse_quote!(mut #numbered);
            },
            FnArg::Receiver(_) => {
                move_self = Some(quote! {()});
            }
        }
    }

    let has_inline = function
        .attrs
        .iter()
        .flat_map(Attribute::parse_meta)
        .any(|meta| meta.path().is_ident("inline"));
    if !has_inline {
        function.attrs.push(parse_quote!(#[inline]));
    }

    let ret = match &function.sig.output {
        ReturnType::Default => quote!(-> ()),
        output @ ReturnType::Type(..) => quote!(#output),
    };
    // Boiler plate...end

    // Let's save the function body before editing it
    let body = function.block;

    // Rebuild the body of the function
    function.block = Box::new(parse_quote!({
        let __result = (move || #ret {
            #move_self
            #(
                let #arg_pat = #arg_val;
            )*

            use holochain_metrics::prelude::*;
            let t = ::std::time::SystemTime::now();
            #body
            let latency = t
                .elapsed()
                .expect("Fail to elapsed time")
                .as_millis();
            let metric = Metric::new(#metric_name, latency as f64);

            // Use a lazily instantiated `LoggerMetricPublisher`
            PUBLISHER.write().unwrap().publish(&metric);

        })();
        __result
    }));

    TokenStream::from(quote!(#function))
}
