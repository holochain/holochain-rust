use crate::workflows::commit::invoke_commit_app_entry;
use crate::workflows::meta::invoke_meta;
use crate::workflows::emit_signal::invoke_emit_signal;
use crate::workflows::capabilities::invoke_commit_capability_claim;
use crate::workflows::capabilities::invoke_commit_capability_grant;
use crate::workflows::keystore::invoke_keystore_get_public_key;
use crate::workflows::keystore::invoke_keystore_sign;
use crate::workflows::keystore::invoke_keystore_derive_key;
use crate::workflows::keystore::invoke_keystore_derive_seed;
use crate::workflows::keystore::invoke_keystore_new_random;
use crate::workflows::keystore::invoke_keystore_list;
use crate::workflows::verify_signature::invoke_verify_signature;
use crate::workflows::sign::invoke_sign_one_time;
use crate::workflows::crypto::invoke_crypto;
use crate::workflows::invoke_remove_link::invoke_remove_link;
use crate::workflows::sleep::invoke_sleep;
use crate::workflows::send::invoke_send;
use crate::workflows::entry_address::invoke_entry_address;
use crate::workflows::query::invoke_query;
use crate::workflows::get_links_count::invoke_get_links_count;
use crate::workflows::get_link_result::invoke_get_links;
use crate::workflows::link_entries::invoke_link_entries;
use crate::workflows::call::invoke_call;
use crate::workflows::init_globals::invoke_init_globals;
use crate::workflows::remove_entry::invoke_remove_entry;
use crate::workflows::update_entry::invoke_update_entry;
use crate::workflows::get_entry_result::invoke_get_entry;
use crate::nucleus::actions::trace_return_wasm_api_function::trace_return_wasm_api_function;
use crate::nucleus::actions::trace_invoke_wasm_api_function::trace_invoke_wasm_api_function;
use crate::workflows::debug::invoke_debug;
use crate::nucleus::WasmApiFnCall;
use holochain_wasm_types::ZomeApiResult;
use crate::wasm_engine::runtime::Runtime;
use core::str::FromStr;

#[macro_export]
macro_rules! link_zome_api {
    (
        $(
            $(#[$meta:meta])*
            $internal_name:literal, $enum_variant:ident, $function_name:path ;
        )*
    ) => {

        // use crate::nucleus::{
        //     actions::{trace_invoke_wasm_api_function::trace_invoke_wasm_api_function, trace_return_wasm_api_function::trace_return_wasm_api_function},
        //     WasmApiFnCall,
        // };
        use $crate::wasm_engine::runtime::WasmCallData;
        use holochain_json_api::json::JsonString;
        use $crate::wasm_engine::Defn;
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
            // cannot test this because PartialEq is not implemented for fns
            #[cfg_attr(tarpaulin, skip)]
            pub fn apply<J: Into<JsonString>>(&self, runtime: &mut Runtime, args: J) -> ZomeApiResult {
                // TODO Implement a proper "abort" function for handling assemblyscript aborts
                // @see: https://github.com/holochain/holochain-rust/issues/324

                match *self {
                    ZomeApiFunction::MissingNo => Ok(()),
                    ZomeApiFunction::Abort => Ok(()),
                    $( ZomeApiFunction::$enum_variant => {
                        if let Ok(context) = runtime.context() {
                            if let WasmCallData::ZomeCall(zome_call_data) = runtime.data.clone() {
                                let zome_api_call = zome_call_data.call;
                                let parameters = runtime.load_json_string_from_args(&args);
                                let hdk_fn_call = WasmApiFnCall { function: self.clone(), parameters };
                                trace_invoke_wasm_api_function(zome_api_call.clone(), hdk_fn_call.clone(), &context);
                                let result = $function_name(runtime, args.into());
                                let hdk_fn_result = Ok(JsonString::from("TODO"));
                                trace_return_wasm_api_function(zome_api_call.clone(), hdk_fn_call, hdk_fn_result, &context);
                                result
                            } else {
                                error!("Can't record zome call hdk invocations for non zome call");
                                $function_name(runtime, args)
                            }
                        } else {
                            error!("Could not get context for runtime");
                            $function_name(runtime, args)
                        }
                    } , )*
                }
            }
        }
    };
}

link_zome_api! {
    /// send debug information to the log
    /// debug(s: String)
    "hc_debug", Debug, invoke_debug;

    /// Commit an app entry to source chain
    /// commit_entry(entry_type: String, entry_value: String) -> Address
    "hc_commit_entry", CommitAppEntry, invoke_commit_app_entry;

    /// Get an app entry from source chain by key (header hash)
    /// get_entry(address: Address) -> Entry
    "hc_get_entry", GetAppEntry, invoke_get_entry;
    "hc_update_entry", UpdateEntry, invoke_update_entry;
    "hc_remove_entry", RemoveEntry, invoke_remove_entry;

    /// Init Zome API Globals
    /// hc_init_globals() -> InitGlobalsOutput
    "hc_init_globals", InitGlobals, invoke_init_globals;

    /// Call a zome function in a different zome or dna via a bridge
    /// hc_call(zome_name: String, cap_token: Address, fn_name: String, args: String);
    "hc_call", Call, invoke_call;

    /// Create a link entry
    "hc_link_entries", LinkEntries, invoke_link_entries;

    /// Retrieve links from the DHT
    "hc_get_links", GetLinks, invoke_get_links;

    //Retrieve link count from DHT
    "hc_get_links_count", GetLinksCount, invoke_get_links_count;

    /// Query the local chain for entries
    "hc_query", Query, invoke_query;

    /// Pass an entry to retrieve its address
    /// the address algorithm is specific to the entry, typically sha256 but can differ
    /// entry_address(entry: Entry) -> Address
    "hc_entry_address", EntryAddress, invoke_entry_address;

    /// Send a message directly to another node
    "hc_send", Send, invoke_send;

    /// Allow a specified amount of time to pass
    "hc_sleep", Sleep, invoke_sleep;

    /// Commit link deletion entry
    "hc_remove_link", RemoveLink, invoke_remove_link;
    //execute cryptographic function
    "hc_crypto",Crypto,invoke_crypto;
    /// Sign a block of data with a one-time key that is then shredded
    "hc_sign_one_time", SignOneTime, invoke_sign_one_time;

    /// Verify that a block of data was signed by a given public key
    "hc_verify_signature", VerifySignature, invoke_verify_signature;

    /// Retrieve a list of identifiers of the secrets in the keystore
    "hc_keystore_list", KeystoreList, invoke_keystore_list;

    /// Create a new random seed Secret in the keystore
    "hc_keystore_new_random", KeystoreNewRandom, invoke_keystore_new_random;

    /// Derive a new seed from an existing seed in the keystore
    "hc_keystore_derive_seed", KeystoreDeriveSeed, invoke_keystore_derive_seed;

    /// Create a new key (signing or encrypting) as derived from an existing seed in the keystore
    "hc_keystore_derive_key", KeystoreDeriveKey, invoke_keystore_derive_key;

    /// Sign a block of data using a key in the keystore
    "hc_keystore_sign", KeystoreSign, invoke_keystore_sign;

    /// Get the public key for a given secret
    "hc_keystore_get_public_key", KeystoreGetPublicKey, invoke_keystore_get_public_key;

    /// Commit a capability grant to the source chain
    "hc_commit_capability_grant", CommitCapabilityGrant, invoke_commit_capability_grant;

    /// Commit a capability grant to the source chain
    "hc_commit_capability_claim", CommitCapabilityClaim, invoke_commit_capability_claim;

    /// Send a DNA defined signal to UIs and other listeners
    "hc_emit_signal", EmitSignal, invoke_emit_signal;

    ///send a meta
    "hc_meta",Meta,invoke_meta;
}
