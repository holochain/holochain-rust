use proc_macro2::TokenStream;
use quote::quote;

pub fn panic_handler() -> TokenStream {
    quote! {
        #[no_mangle]
        pub extern "C" fn __install_panic_handler() -> () {
            use hdk::{api::debug, holochain_json_api::json::RawString};
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
