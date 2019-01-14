//! This file contains the define_zome! macro, and smaller helper macros.

#[doc(hidden)]
#[macro_export]
macro_rules! load_json {
    ($encoded_allocation_of_input:ident) => {{
        let encoded_bits: $crate::holochain_wasm_utils::holochain_core_types::error::RibosomeEncodingBits: $encoded_allocation_of_input;
        let ribosome_allocation = $crate::holochain_wasm_utils::holochain_core_types::error::RibosomeEncodedAllocation(encoded_bits);
        let allocation_result = match $crate::holochain_wasm_utils::memory::allocation::WasmAllocation::try_from(ribosome_allocation);
        let allocation = match allocation_result {
            Ok(allocation) => allocation,
            Err(_) => return return_code_for_allocation_result(allocation_result),
        };
        let string = allocation.read_string();

        let maybe_input = $crate::holochain_wasm_utils::memory_serialization::load_json(
            $encoded_allocation_of_input,
        );
        if let Err(hc_err) = maybe_input {
            return $crate::global_fns::write_json(hc_err);
        }
        maybe_input
    }};
}
#[doc(hidden)]
#[macro_export]
macro_rules! load_string {
    ($encoded_allocation_of_input:ident) => {{
        let maybe_input = $crate::holochain_wasm_utils::memory_serialization::load_string(
            $encoded_allocation_of_input,
        );
        if let Err(hc_err) = maybe_input {
            return $crate::global_fns::store_and_return_output(hc_err);
        }
        maybe_input
    }};
}

/// Every Zome must utilize the `define_zome`
/// macro in the main library file in their Zome.
/// The `define_zome` macro has 4 component parts:
/// 1. entries: an array of [ValidatingEntryType](entry_definition/struct.ValidatingEntryType.html) as returned by using the [entry](macro.entry.html) macro
/// 2. genesis: `genesis` is a callback called by Holochain to every Zome implemented within a DNA.
///     It gets called when a new agent is initializing an instance of the DNA for the first time, and
///     should return `Ok` or an `Err`, depending on whether the agent can join the network or not.
/// 3. receive (optional): `receive` is a callback called by Holochain when another agent on a hApp has initiated a node-to-node direct message.
///     That node-to-node message is initiated via the [**send** function of the API](api/fn.send.html), which is where you can read further about use of `send` and `receive`.
///     `receive` is optional to include, based on whether you use `send` anywhere in the code.
/// 4. functions: `functions` is divided up into `capabilities`, which specify who can access those functions.
///     `functions` must be a tree structure where the first children are `capabilities`
///     and the children of those `capabilities` are actual function definitions.
/// # Examples
///
/// ```rust
/// # #![feature(try_from)]
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # extern crate boolinator;
/// # extern crate holochain_core_types;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_core_types::entry::entry_type::AppEntryType;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use boolinator::Boolinator;
/// use hdk::error::ZomeApiResult;
/// use holochain_core_types::{
///     cas::content::Address,
///     dna::entry_types::Sharing,
/// };
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_query(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_send(_: u32) -> u32 { 0 }
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson)]
/// pub struct Post {
///     content: String,
///     date_created: String,
/// }
///
/// fn handle_post_address(content: String) -> ZomeApiResult<Address> {
///     let post_entry = Entry::App("post".into(), Post {
///         content,
///         date_created: "now".into(),
///     }.into());
///
///     hdk::entry_address(&post_entry)
/// }
///
/// define_zome! {
///     entries: [
///         entry!(
///             name: "post",
///             description: "",
///             sharing: Sharing::Public,
///             native_type: Post,
///
///             validation_package: || {
///                 hdk::ValidationPackageDefinition::ChainFull
///             },
///
///             validation: |post: Post, _ctx: hdk::ValidationData| {
///                 (post.content.len() < 280)
///                     .ok_or_else(|| String::from("Content too long"))
///             }
///         )
///     ]
///
///     genesis: || {
///         Ok(())
///     }
///
///     receive: |payload| {
///       // just return what was received, but modified
///       format!("Received: {}", payload)
///     }
///
///     functions: {
///         // "main" is the name of the capability
///         // "Public" is the access setting of the capability
///         main (Public) {
///             // the name of this function, "post_address" is the
///             // one to give while performing a `call` method to this function.
///             // the name of the handler function must be different than the
///             // name of the Zome function.
///             post_address: {
///                 inputs: |content: String|,
///                 outputs: |post: ZomeApiResult<Address>|,
///                 handler: handle_post_address
///             }
///         }
///     }
/// }
///
/// # }
/// ```
#[macro_export]
macro_rules! define_zome {
    (
        entries : [
            $( $entry_expr:expr ),*
        ]

        genesis : || {
            $genesis_expr:expr
        }

        $(
            receive : |$receive_param:ident| {
                $receive_expr:expr
            }
        )*

        functions : {
            $(
                $cap:ident ( $vis:ident ) {
                    $(
                        $zome_function_name:ident : {
                            inputs: | $( $input_param_name:ident : $input_param_type:ty ),* |,
                            outputs: | $( $output_param_name:ident : $output_param_type:ty ),* |,
                            handler: $handler_path:path
                        }
                    )+
                }
            )*
        }

    ) => {
        #[no_mangle]
        #[allow(unused_variables)]
        pub extern "C" fn zome_setup(zd: &mut $crate::meta::ZomeDefinition) {
            $(
                zd.define($entry_expr);
            )*
        }

        #[no_mangle]
        pub extern "C" fn genesis(encoded_allocation_of_input: u32) -> u32 {
            $crate::global_fns::init_global_memory(encoded_allocation_of_input);

            fn execute() -> Result<(), String> {
                $genesis_expr
            }

            match execute() {
                Ok(_) => 0,
                Err(e) => $crate::global_fns::store_and_return_output($crate::holochain_wasm_utils::holochain_core_types::json::RawString::from(e)),
            }
        }

        $(
            #[no_mangle]
            pub extern "C" fn receive(encoded_allocation_of_input: u32) -> u32 {
                $crate::global_fns::init_global_memory(encoded_allocation_of_input);

                // Deserialize input
                let input = load_string!(encoded_allocation_of_input).unwrap();

                fn execute(payload: String) -> String {
                    let $receive_param = payload;
                    $receive_expr
                }

                $crate::global_fns::store_and_return_output(execute(input))
            }
        )*

        use $crate::holochain_core_types::dna::capabilities::Capability;
        use std::collections::HashMap;

        #[no_mangle]
        #[allow(unused_imports)]
        pub fn __list_capabilities() -> $crate::holochain_core_types::dna::zome::ZomeCapabilities {

            use $crate::holochain_core_types::dna::capabilities::{Capability, CapabilityType, FnParameter, FnDeclaration};
            use std::collections::BTreeMap;

            let return_value: $crate::holochain_core_types::dna::zome::ZomeCapabilities = {
                let mut cap_map = BTreeMap::new();

                $(
                    {
                        let mut capability = Capability::new(CapabilityType::$vis);
                        capability.functions = vec![
                            $(
                                FnDeclaration {
                                    name: stringify!($zome_function_name).into(),
                                    inputs: vec![
                                        $(
                                            FnParameter::new(stringify!($input_param_name), stringify!($input_param_type))
                                        ),*
                                    ],
                                    outputs: vec![
                                        $(
                                            FnParameter::new(stringify!($output_param_name), stringify!($output_param_type))
                                        ),*
                                    ]
                                }

                            ),+
                        ];

                        cap_map.insert(stringify!($cap).into(), capability);
                    }
                ),*

                cap_map
            };

            return_value
        }

        $(
            $(
                #[no_mangle]
                pub extern "C" fn $zome_function_name(encoded_allocation_of_input: u32) -> u32 {
                    $crate::global_fns::init_global_memory(encoded_allocation_of_input);

                    // Macro'd InputStruct
                    #[derive(Deserialize, Debug)]
                    struct InputStruct {
                        $($input_param_name : $input_param_type),*
                    }

                    // Deserialize input
                    let maybe_input = load_json!(encoded_allocation_of_input);
                    let input: InputStruct = maybe_input.unwrap();

                    // Macro'd function body
                    fn execute (params: InputStruct) -> $( $output_param_type )* {
                        let InputStruct { $($input_param_name),* } = params;

                        $handler_path($($input_param_name),*)
                    }

                    $crate::global_fns::store_and_return_output(execute(input))
                }
            )+
        )*
    };
}
