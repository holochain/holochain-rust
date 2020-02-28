use core::str::FromStr;

#[macro_export]
macro_rules! link_zome_api {
    (
        $(
            $(#[$meta:meta])*
            $internal_name:literal, $enum_variant:ident ;
        )*
    ) => {

        // use crate::nucleus::{
        //     actions::{trace_invoke_wasm_api_function::trace_invoke_wasm_api_function, trace_return_wasm_api_function::trace_return_wasm_api_function},
        //     WasmApiFnCall,
        // };
        // use $crate::wasm_engine::runtime::WasmCallData;
        // use holochain_json_api::json::JsonString;
        use $crate::wasm_engine::Defn;
        // use std::convert::TryFrom;
        use num;

        /// Enumeration of all the Zome Functions known and usable in Zomes.
        /// Enumeration can convert to str.
        #[repr(usize)]
        #[derive(FromPrimitive, Clone, Hash, Debug, PartialEq, Eq, Serialize)]
        pub enum ZomeApiFunction {
            /// Error index for unimplemented functions
            MissingNo = 0,

            /// Abort is a way to receive useful debug info from
            /// assemblyscript memory allocators
            /// message: mem address in the wasm memory for an error message
            /// filename: mem address in the wasm memory for a filename
            /// line: line number
            /// column: column number
            Abort,

            $(
                $(#[$meta])*
                $enum_variant
            ),*
        }

        impl Defn for ZomeApiFunction {
            fn as_str(&self) -> &'static str {
                match *self {
                    ZomeApiFunction::MissingNo => "",
                    ZomeApiFunction::Abort => "abort",
                    $(ZomeApiFunction::$enum_variant => $internal_name),*
                }
            }

            fn str_to_index(s: &str) -> usize {
                ZomeApiFunction::from_str(s).unwrap_or(ZomeApiFunction::MissingNo) as usize
            }

            fn from_index(i: usize) -> Self {
                num::FromPrimitive::from_usize(i).unwrap_or(ZomeApiFunction::MissingNo)
            }
        }

        impl FromStr for ZomeApiFunction {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "abort" => Ok(ZomeApiFunction::Abort),
                    $($internal_name => Ok(ZomeApiFunction::$enum_variant),)*
                    _ => Err("Cannot convert string to ZomeApiFunction"),
                }
            }
        }

        impl ZomeApiFunction {
            // // cannot test this because PartialEq is not implemented for fns
            // #[cfg_attr(tarpaulin, skip)]
            // pub fn apply<J: TryFrom<JsonString>>(&self, runtime: &mut Runtime, guest_allocation_ptr: $crate::holochain_wasmer_host::AllocationPtr) -> ZomeApiResult {
            //     // TODO Implement a proper "abort" function for handling assemblyscript aborts
            //     // @see: https://github.com/holochain/holochain-rust/issues/324
            //     let guest_bytes = $crate::holochain_wasmer_host::guest::read_from_allocation_ptr(runtime.wasm_instance.context_mut(), guest_allocation_ptr)?;
            //     let parameters = JsonString::from_bytes(guest_bytes);
            //
            //     match *self {
            //         ZomeApiFunction::MissingNo => Ok(holochain_wasmer_host::json::to_allocation_ptr(().into())),
            //         ZomeApiFunction::Abort => Ok(holochain_wasmer_host::json::to_allocation_ptr(().into())),
            //         $( ZomeApiFunction::$enum_variant => {
            //             if let Ok(context) = runtime.context() {
            //                 if let WasmCallData::ZomeCall(zome_call_data) = runtime.data.clone() {
            //                     let zome_api_call = zome_call_data.call;
            //                     // let parameters = runtime.load_json_string_from_args(&args);
            //                     let hdk_fn_call = WasmApiFnCall { function: self.clone(), parameters };
            //                     trace_invoke_wasm_api_function(zome_api_call.clone(), hdk_fn_call.clone(), &context);
            //                     let result = $function_name(runtime, parameters.try_into()?);
            //                     let hdk_fn_result = Ok(JsonString::from("TODO"));
            //                     trace_return_wasm_api_function(zome_api_call.clone(), hdk_fn_call, hdk_fn_result, &context);
            //                     match result {
            //                         Ok(v) => Ok(holochain_wasmer_host::json::to_allocation_ptr(v.try_into()?)),
            //                         Err(e) => Err(WasmError::from(e)),
            //                     }
            //                 } else {
            //                     error!("Can't record zome call hdk invocations for non zome call");
            //                     match $function_name(runtime, parameters.try_into()?) {
            //                         Ok(v) => Ok(holochain_wasmer_host::json::to_allocation_ptr(v.try_into()?)),
            //                         Err(e) => Err(WasmError::from(e)),
            //                     }
            //                 }
            //             } else {
            //                 error!("Could not get context for runtime");
            //                 match $function_name(runtime, parameters.try_into()?) {
            //                     Ok(v) => Ok(holochain_wasmer_host::json::to_allocation_ptr(v.into())),
            //                     Err(e) => Err(WasmError::from(e)),
            //                 }
            //             }
            //         } , )*
            //     }
            // }
        }
    };
}

link_zome_api! {
    /// send debug information to the log
    /// debug(s: String)
    "hc_debug", Debug;

    /// Commit an app entry to source chain
    /// commit_entry(entry_type: String, entry_value: String) -> Address
    "hc_commit_entry", CommitAppEntry;

    /// Get an app entry from source chain by key (header hash)
    /// get_entry(address: Address) -> Entry
    "hc_get_entry", GetAppEntry;
    "hc_update_entry", UpdateEntry;
    "hc_remove_entry", RemoveEntry;

    /// Init Zome API Globals
    /// hc_init_globals() -> InitGlobalsOutput
    "hc_init_globals", InitGlobals;

    /// Call a zome function in a different zome or dna via a bridge
    /// hc_call(zome_name: String, cap_token: Address, fn_name: String, args: String);
    "hc_call", Call;

    /// Create a link entry
    "hc_link_entries", LinkEntries;

    /// Retrieve links from the DHT
    "hc_get_links", GetLinks;

    //Retrieve link count from DHT
    "hc_get_links_count", GetLinksCount;

    /// Query the local chain for entries
    "hc_query", Query;

    /// Pass an entry to retrieve its address
    /// the address algorithm is specific to the entry, typically sha256 but can differ
    /// entry_address(entry: Entry) -> Address
    "hc_entry_address", EntryAddress;

    /// Send a message directly to another node
    "hc_send", Send;

    /// Allow a specified amount of time to pass
    "hc_sleep", Sleep;

    /// Commit link deletion entry
    "hc_remove_link", RemoveLink;
    //execute cryptographic function
    "hc_crypto", Crypto;
    /// Sign a block of data with a one-time key that is then shredded
    "hc_sign_one_time", SignOneTime;

    /// Verify that a block of data was signed by a given public key
    "hc_verify_signature", VerifySignature;

    /// Retrieve a list of identifiers of the secrets in the keystore
    "hc_keystore_list", KeystoreList;

    /// Create a new random seed Secret in the keystore
    "hc_keystore_new_random", KeystoreNewRandom;

    /// Derive a new seed from an existing seed in the keystore
    "hc_keystore_derive_seed", KeystoreDeriveSeed;

    /// Create a new key (signing or encrypting) as derived from an existing seed in the keystore
    "hc_keystore_derive_key", KeystoreDeriveKey;

    /// Sign a block of data using a key in the keystore
    "hc_keystore_sign", KeystoreSign;

    /// Get the public key for a given secret
    "hc_keystore_get_public_key", KeystoreGetPublicKey;

    /// Commit a capability grant to the source chain
    "hc_commit_capability_grant", CommitCapabilityGrant;

    /// Commit a capability grant to the source chain
    "hc_commit_capability_claim", CommitCapabilityClaim;

    /// Send a DNA defined signal to UIs and other listeners
    "hc_emit_signal", EmitSignal;

    ///send a meta
    "hc_meta", Meta;
}
