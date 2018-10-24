#[macro_export]
macro_rules! load_json {
    ($encoded_allocation_of_input:ident) => {{
        let maybe_input =
            ::hdk::holochain_wasm_utils::memory_serialization::load_json($encoded_allocation_of_input);
        if let Err(_) = maybe_input {
            return ::hdk::holochain_wasm_utils::holochain_core_types::error::RibosomeErrorCode::ArgumentDeserializationFailed
                as u32;
        }
        maybe_input
    }};
}

/// A macro for easily writing zome functions
///
/// # Examples
/// ```
/// # #[macro_use] extern crate hdk;
/// # extern crate holochain_wasm_utils;
/// # extern crate serde;
/// # extern crate serde_json;
/// # #[macro_use] extern crate serde_derive;
/// # use hdk::globals::G_MEM_STACK;
/// # use holochain_wasm_utils::holochain_core_types::error::RibosomeReturnCode;
///
/// # // Adding empty hc_init_globals() so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: u32) -> u32 { 0 }
///
/// # fn main() {
/// #[derive(Serialize)]
/// struct CreatePostResponse {
///     author: String,
/// }
///
/// zome_functions! {
///     create_post: |author: String, content: String| {
///
///         // ..snip..
///
///         CreatePostResponse { author: author }
///     }
/// }
/// # }
/// ```
///
#[macro_export]
macro_rules! zome_functions {
    (
        $($func_name:ident : | $($param:ident : $param_type:ty),* | $main_block:expr)+
    ) => (

        $(
            #[no_mangle]
            pub extern "C" fn $func_name(encoded_allocation_of_input: u32) -> u32 {

                ::hdk::global_fns::init_global_memory(encoded_allocation_of_input);

                // Macro'd InputStruct
                #[derive(Deserialize)]
                struct InputStruct {
                    $($param : $param_type),*
                }

                // Deserialize input
                let maybe_input = load_json!(encoded_allocation_of_input);
                let input: InputStruct = maybe_input.unwrap();

                // Macro'd function body
                fn execute(params: InputStruct) -> impl ::serde::Serialize {
                    let InputStruct { $($param),* } = params;
                    $main_block
                }

                // Execute inner function
                let output_obj = execute(input);

                ::hdk::global_fns::store_and_return_output(output_obj)
            }
        )+
    );
}