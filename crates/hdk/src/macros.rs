//! This file contains the define_zome! macro, and smaller helper macros.

/// Every Zome must utilize the `define_zome`
/// macro in the main library file in their Zome.
/// The `define_zome` macro has 4 component parts:
/// 1. entries: an array of [ValidatingEntryType](entry_definition/struct.ValidatingEntryType.html) as returned by using the [entry](macro.entry.html) macro
/// 2. init: `init` is a callback called by Holochain to every Zome implemented within a DNA.
///     It gets called when a new agent is initializing an instance of the DNA for the first time, and
///     should return `Ok` or an `Err`, depending on whether the agent can join the network or not.
/// 3. receive (optional): `receive` is a callback called by Holochain when another agent on a hApp has initiated a node-to-node direct message.
///     That node-to-node message is initiated via the [**send** function of the API](api/fn.send.html), which is where you can read further about use of `send` and `receive`.
///     `receive` is optional to include, based on whether you use `send` anywhere in the code.
/// 4. functions:
///     `functions` declares all the zome's functions with their input/output signatures
/// # Examples
///
/// ```rust
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
/// # extern crate holochain_json_derive;
/// # extern crate holochain_json_api;
/// # extern crate holochain_persistence_api;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_core_types::entry::entry_type::AppEntryType;
/// # use holochain_json_api::{error::JsonError, json::JsonString};
/// # use holochain_core_types::error::HolochainError;
/// # use boolinator::Boolinator;
/// use hdk::error::ZomeApiResult;
/// use holochain_core_types::{
///     dna::entry_types::Sharing,
///     validation::EntryValidationData
/// };
/// # use holochain_persistence_api::cas::content::Address;
/// # use holochain_core_types::error::AllocationPtr;
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_query(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: AllocationPtr) -> AllocationPtr { ret!(())); }
/// # #[no_mangle]
/// # pub fn hc_send(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_sleep(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_debug(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_call(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// #[no_mangle]
/// # pub fn hc_crypto(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// #[no_mangle]
/// # pub fn hc_meta(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_sign_one_time(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_verify_signature(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_get_links(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_get_links_count(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_link_entries(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_remove_link(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_keystore_list(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_keystore_new_random(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_seed(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_key(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_keystore_sign(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_keystore_get_public_key(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_commit_capability_grant(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_commit_capability_claim(_: AllocationPtr) -> AllocationPtr { ret!(()); }
/// # #[no_mangle]
/// # pub fn hc_emit_signal(_: AllocationPtr) -> AllocationPtr { ret!(()); }
///
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
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
///
///             validation_package: || {
///                 hdk::ValidationPackageDefinition::ChainFull
///             },
///
///             validation: |validation_data: hdk::EntryValidationData<Post>| {
///              match validation_data
///              {
///              EntryValidationData::Create{entry:test_entry,validation_data:_} =>
///              {
///
///
///                        (test_entry.content != "FAIL")
///                        .ok_or_else(|| "FAIL content is not allowed".to_string())
///                }
///                _ =>
///                 {
///                      Err("Failed to validate with wrong entry type".to_string())
///                }
///         }}
///
///         )
///     ]
///
///     init: || {
///         Ok(())
///     }
///
///     validate_agent: |validation_data : EntryValidationData::<AgentId>| {
///         Ok(())
///     }
///
///     receive: |from, payload| {
///       // just return what was received, but modified
///       format!("Received: {} from {}", payload, from)
///     }
///
///     functions: [
///             // the name of this function, "post_address" is the
///             // one to give while performing a `call` method to this function.
///             // the name of the handler function must be different than the
///             // name of the Zome function.
///             post_address: {
///                 inputs: |content: String|,
///                 outputs: |post: ZomeApiResult<Address>|,
///                 handler: handle_post_address
///             }
///     ]
///
///     // trait named "hc_public" will grant public access to all its functions
///     traits: {
///         hc_public [post_address]
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

        init : || {
            $init_expr:expr
        }


        validate_agent: |$agent_validation_param:ident : EntryValidationData::<AgentId>| {
            $agent_validation_expr:expr
        }


        $(
            receive : |$receive_from:ident, $receive_param:ident| {
                $receive_expr:expr
            }
        )*

        functions : [
            $(
                        $zome_function_name:ident : {
                            inputs: | $( $input_param_name:ident : $input_param_type:ty ),* |,
                            outputs: | $( $output_param_name:ident : $output_param_type:ty ),* |,
                            handler: $handler_path:path
                        }
            )*
        ]

        traits : {
                $(
                    $trait:ident [
                        $($trait_fn:ident),*
                    ]
                )*
            }


    ) => {
        #[no_mangle]
        #[allow(unused_variables)]
        pub extern "C" fn zome_setup(zd: &mut $crate::meta::ZomeDefinition) {
            $(
                zd.define($entry_expr);
            )*

            let validator = Box::new(|validation_data: $crate::holochain_core_types::validation::EntryValidationData<hdk::holochain_core_types::agent::AgentId>| {
                let $agent_validation_param = validation_data;
                let result: $crate::holochain_core_types::validation::ValidationResult = $agent_validation_expr;
                result
            });
            zd.define_agent_validator(validator);
        }

        #[no_mangle]
        pub extern "C" fn init(host_allocation_ptr: holochain_wasmer_guest::AllocationPtr) -> holochain_wasmer_guest::AllocationPtr {

            fn execute() -> $crate::holochain_core_types::callback::CallbackResult {
                $init_expr
            }

            ret!(execute());
        }

        $(
            #[no_mangle]
            pub extern "C" fn receive(host_allocation_ptr: holochain_wasmer_guest::AllocationPtr) -> holochain_wasmer_guest::AllocationPtr {
                let input: $crate::holochain_wasm_types::receive::ReceiveParams = holochain_wasmer_guest::host_args!(host_allocation_ptr);

                fn execute(input: $crate::holochain_wasm_types::receive::ReceiveParams ) -> String {
                    let $receive_param = input.payload;
                    let $receive_from = input.from;
                    $receive_expr
                }

                ret!(WasmResult::Ok(JsonString::from_json(&execute(input))));
            }
        )*

        use std::collections::HashMap;

        #[no_mangle]
        #[allow(unused_imports)]
        pub fn __list_traits() -> $crate::holochain_core_types::dna::zome::ZomeTraits {

            use $crate::holochain_core_types::dna::{
                fn_declarations::{FnParameter, FnDeclaration, TraitFns},
            };

            use std::collections::BTreeMap;

            let return_value: $crate::holochain_core_types::dna::zome::ZomeTraits = {
                let mut traitfns_map = BTreeMap::new();

                $(
                    {
                        let mut traitfns = TraitFns::new();
                        traitfns.functions = vec![
                            $(
                                stringify!($trait_fn).into()
                            ),*
                        ];

                        traitfns_map.insert(stringify!($trait).into(), traitfns);
                    }
                ),*

                traitfns_map
            };

            return_value
        }

        #[no_mangle]
        #[allow(unused_imports)]
        pub fn __list_functions() -> $crate::holochain_core_types::dna::zome::ZomeFnDeclarations {

            use $crate::holochain_core_types::dna::fn_declarations::{FnParameter, FnDeclaration};

            let return_value: $crate::holochain_core_types::dna::zome::ZomeFnDeclarations = {
                vec![

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
                    ),*

                ]
            };

            return_value
        }


        #[no_mangle]
        pub extern "C" fn __install_panic_handler() -> () {
            use $crate::{api::debug, holochain_json_api::json::RawString};
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


        $(
                #[no_mangle]
                pub extern "C" fn $zome_function_name(host_allocation_ptr: holochain_wasmer_guest::AllocationPtr) -> holochain_wasmer_guest::AllocationPtr {
                    // Macro'd InputStruct
                    #[derive(Deserialize, Serialize, Debug, $crate::holochain_json_derive::DefaultJson)]
                    struct InputStruct {
                        $($input_param_name : $input_param_type),*
                    }

                    // Deserialize input
                    let input: InputStruct = holochain_wasmer_guest::host_args!(host_allocation_ptr);

                    // Macro'd function body
                    fn execute (params: InputStruct) -> $( $output_param_type )* {
                        let InputStruct { $($input_param_name),* } = params;

                        $handler_path($($input_param_name),*)
                    }

                    ret!(WasmResult::Ok(execute(input).into()));
                }
        )*
    };
}
