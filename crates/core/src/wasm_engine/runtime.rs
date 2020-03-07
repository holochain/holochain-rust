use crate::{
    context::Context,
    nucleus::{CallbackFnCall, ZomeFnCall},

};
use holochain_json_api::json::JsonString;
use holochain_core_types::error::HolochainError;
use std::{fmt, sync::Arc};
use wasmer_runtime::{error::RuntimeError, Instance, imports, func, instantiate};
use holochain_wasmer_host::WasmError;
use wasmer_runtime::Ctx;
use crate::workflows::debug::debug_workflow;
use holochain_wasm_types::ZomeApiResult;
use std::convert::TryInto;
use crate::workflows::get_links_count::get_link_result_count_workflow;
use crate::workflows::commit::commit_app_entry_workflow;
use crate::workflows::get_entry_result::get_entry_result_workflow;
use crate::workflows::update_entry::update_entry_workflow;
use crate::workflows::remove_entry::remove_entry_workflow;
use crate::workflows::init_globals::init_globals_workflow;
use crate::workflows::get_link_result::get_link_result_workflow;
use crate::workflows::meta::meta_workflow;
use crate::workflows::emit_signal::emit_signal_workflow;
use crate::workflows::sleep::sleep_workflow;
use crate::workflows::verify_signature::verify_signature_workflow;
use crate::workflows::capabilities::commit_capability_grant_workflow;
use crate::workflows::capabilities::commit_capability_claim_workflow;
use crate::workflows::keystore::keystore_list_workflow;
use crate::workflows::keystore::keystore_get_public_key_workflow;
use crate::workflows::keystore::keystore_sign_workflow;
use crate::workflows::keystore::keystore_derive_key_workflow;
use crate::workflows::keystore::keystore_derive_seed_workflow;
use crate::workflows::keystore::keystore_new_random_workflow;
use crate::workflows::sign::sign_one_time_workflow;
use crate::workflows::remove_link_wasm::remove_link_wasm_workflow;
use crate::workflows::send::send_workflow;
use crate::workflows::entry_address::entry_address_workflow;
use crate::workflows::query::query_workflow;
use crate::workflows::call::call_workflow;
use crate::workflows::link_entries::link_entries_workflow;
use crate::workflows::crypto::decrypt_workflow;
use crate::workflows::sign::sign_workflow;
use holochain_wasm_types::WasmResult;

#[derive(Clone)]
pub struct ZomeCallData {
    /// Context of Holochain. Required for operating.
    pub context: Arc<Context>,
    /// The zome function call that initiated the Ribosome.
    pub call: ZomeFnCall,
}

#[derive(Clone)]
pub struct CallbackCallData {
    /// Context of Holochain. Required for operating.
    pub context: Arc<Context>,
    /// The callback function call that initiated the Ribosome.
    pub call: CallbackFnCall,
}

#[derive(Clone)]
pub enum WasmCallData {
    ZomeCall(ZomeCallData),
    CallbackCall(CallbackCallData),
    DirectCall(String, Arc<Vec<u8>>),
}

#[derive(Debug)]
struct BadCallError(String);
impl fmt::Display for BadCallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad calling context: {}", self.0)
    }
}

// impl HostError for BadCallError {}

// // #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl WasmCallData {
    pub fn new_zome_call(context: Arc<Context>, call: ZomeFnCall) -> Self {
        WasmCallData::ZomeCall(ZomeCallData { context, call })
    }

    pub fn new_callback_call(context: Arc<Context>, call: CallbackFnCall) -> Self {
        WasmCallData::CallbackCall(CallbackCallData { context, call })
    }

    pub fn fn_name(&self) -> String {
        match self {
            WasmCallData::ZomeCall(data) => data.call.fn_name.clone(),
            WasmCallData::CallbackCall(data) => data.call.fn_name.clone(),
            WasmCallData::DirectCall(name, _) => name.to_string(),
        }
    }

    pub fn context(&self) -> Result<Arc<Context>, HolochainError> {
        match &self {
            WasmCallData::ZomeCall(ref data) => Ok(data.context.clone()),
            WasmCallData::CallbackCall(ref data) => Ok(data.context.clone()),
            _ => Err(HolochainError::ErrorGeneric(format!("context data: {:?}", &self))),
        }
    }

    pub fn zome_call_data(&self) -> Result<ZomeCallData, RuntimeError> {
        match &self {
            WasmCallData::ZomeCall(ref data) => Ok(data.clone()),
            _ => Err(RuntimeError::Trap {
                msg: format!("zome_call_data: {:?}", &self).into_boxed_str(),
            }),
        }
    }

    pub fn callback_call_data(&self) -> Result<CallbackCallData, RuntimeError> {
        match &self {
            WasmCallData::CallbackCall(ref data) => Ok(data.clone()),
            _ => Err(RuntimeError::Trap {
                msg: format!("callback_call_data: {:?}", &self).into_boxed_str(),
            }),
        }
    }

    pub fn call_data(&self) -> Result<CallData, RuntimeError> {
        match &self {
            WasmCallData::ZomeCall(ref data) => Ok(CallData {
                context: data.context.clone(),
                zome_name: data.call.zome_name.clone(),
                fn_name: data.call.fn_name.clone(),
                parameters: data.call.parameters.clone(),
            }),
            WasmCallData::CallbackCall(ref data) => Ok(CallData {
                context: data.context.clone(),
                zome_name: data.call.zome_name.clone(),
                fn_name: data.call.fn_name.clone(),
                parameters: data.call.parameters.clone(),
            }),
            _ => Err(RuntimeError::Trap {
                msg: format!("call_data: {:?}", &self).into_boxed_str(),
            }),
        }
    }

    pub fn instance(&self) -> Result<Instance, HolochainError> {
        let arc = std::sync::Arc::new(self.clone());

        macro_rules! invoke_workflow_trace {
            ( $context:ident, $trace_span:literal, $trace_tag:literal, $args:ident ) => {{
                let span = $context
                    .tracer
                    .span(format!("hdk {}", $trace_span))
                    .tag(ht::Tag::new(
                        $trace_tag,
                        format!("{:?}", $args),
                    ))
                    .start()
                    .into();
                let _spanguard = $crate::ht::push_span(span);
            }}
        }

        macro_rules! invoke_workflow_block_and_allocate {
            ( $workflow:ident, $context:ident, $args:ident ) => {{
                Ok(holochain_wasmer_host::json::to_allocation_ptr(
                    $context.block_on(
                        $workflow(std::sync::Arc::clone(&$context), &$args)
                    ).map_err(|e| WasmError::Zome(e.to_string()))?.into()
                ))
            }}
        }

        macro_rules! invoke_workflow {
            ( $trace_span:literal, $trace_tag:literal, $workflow:ident ) => {{
                let closure_arc = std::sync::Arc::clone(&arc);
                move |ctx: &mut Ctx, guest_allocation_ptr: holochain_wasmer_host::AllocationPtr| -> ZomeApiResult {
                    let guest_bytes = holochain_wasmer_host::guest::read_from_allocation_ptr(ctx, guest_allocation_ptr)?;
                    let guest_json = JsonString::from_bytes(guest_bytes);
                    println!("invoke_workflow: {}", &guest_json);
                    let context = std::sync::Arc::clone(&closure_arc.context().map_err(|_| WasmError::Unspecified )?);

                    // in general we will have more luck tracing json than arbitrary structs
                    invoke_workflow_trace!(context, $trace_span, $trace_tag, guest_json);
                    let args = guest_json.try_into()?;
                    invoke_workflow_block_and_allocate!($workflow, context, args)
                }
            }}
        }

        let wasm_imports = imports! {
            "env" => {
                "__import_allocation" => wasmer_runtime::func!(holochain_wasmer_host::import::__import_allocation),
                "__import_bytes" => wasmer_runtime::func!(holochain_wasmer_host::import::__import_bytes),

                // send debug information to the log
                // debug(s: String)
                "hc_debug" => func!(invoke_workflow!("debug_workflow", "WasmString", debug_workflow)),

                // Commit an app entry to source chain
                // commit_entry(entry_type: String, entry_value: String) -> Address
                "hc_commit_entry" => func!(invoke_workflow!("commit_app_entry_workflow", "CommitEntryArgs", commit_app_entry_workflow)),

                // Get an app entry from source chain by key (header hash)
                // get_entry(address: Address) -> Entry
                "hc_get_entry" => func!(invoke_workflow!("get_entry_result_workflow", "GetEntryArgs", get_entry_result_workflow)),
                "hc_update_entry" => func!(invoke_workflow!("update_entry_workflow", "UpdateEntryArgs", update_entry_workflow)),
                "hc_remove_entry" => func!(invoke_workflow!("remove_entry_workflow", "Address", remove_entry_workflow)),

                // Init Zome API Globals
                // hc_init_globals() -> InitGlobalsOutput
                // there is no input from the guest for input_globals_workflow
                // instead it needs direct access to the wasm call data
                "hc_init_globals" => func!({
                    let closure_arc = std::sync::Arc::clone(&arc);
                    move |_: &mut Ctx, _: holochain_wasmer_host::AllocationPtr| -> ZomeApiResult {
                        let context = Arc::clone(&closure_arc.context().map_err(|_| WasmError::Unspecified )?);
                        let args = Arc::clone(&closure_arc);
                        invoke_workflow_trace!(context, "init_globals_workflow", "WasmCallData", args);
                        invoke_workflow_block_and_allocate!(init_globals_workflow, context, args)
                    }
                }),

                // Call a zome function in a different zome or dna via a bridge
                // hc_call(zome_name: String, cap_token: Address, fn_name: String, args: String);
                // call_workflow is weird in that it needs BOTH input from the guest AND direct
                // access to the wasm call data
                // this creates a non-standard workflow function signature with 3 args
                // wasm call data cannot be rolled into the input arg as it must be provided by the
                // host while the input data must be provided by the guest
                "hc_call" => func!({
                    let closure_arc = std::sync::Arc::clone(&arc);
                    move |ctx: &mut Ctx, guest_allocation_ptr: holochain_wasmer_host::AllocationPtr| -> ZomeApiResult {
                        println!("hc_call invoke");
                        let guest_bytes = holochain_wasmer_host::guest::read_from_allocation_ptr(ctx, guest_allocation_ptr)?;
                        let guest_json = JsonString::from_bytes(guest_bytes);
                        println!("hc_call guest_json {:?}", &guest_json);
                        let context = std::sync::Arc::clone(&closure_arc.context().map_err(|_| WasmError::Unspecified )?);

                        invoke_workflow_trace!(context, "call_workflow", "ZomeFnCallArgs", guest_json);
                        let args = guest_json.try_into()?;
                        println!("hc_call args {:?}", &args);
                        Ok(holochain_wasmer_host::json::to_allocation_ptr(
                            {
                                let result: WasmResult = context.block_on(
                                    call_workflow(Arc::clone(&context), Arc::clone(&closure_arc), &args)
                                ).map_err(|e| WasmError::Zome(e.to_string()))?;
                                println!("hc_call r: {:?}", &result);
                                JsonString::from(result)
                            }
                        ))
                    }
                }),

                // Create a link entry
                "hc_link_entries" => func!(invoke_workflow!("link_entries_workflow", "LinkEntriesArgs", link_entries_workflow)),

                /// Retrieve links from the DHT
                "hc_get_links" => func!(invoke_workflow!("get_link_result_workflow", "GetLinksArgs", get_link_result_workflow)),

                //Retrieve link count from DHT
                "hc_get_links_count" => func!(invoke_workflow!("get_link_result_count_workflow", "GetLinksArgs", get_link_result_count_workflow)),

                // Query the local chain for entries
                "hc_query" => func!(invoke_workflow!("query_workflow", "QueryArgs", query_workflow)),

                // Pass an entry to retrieve its address
                // the address algorithm is specific to the entry, typically sha256 but can differ
                // entry_address(entry: Entry) -> Address
                "hc_entry_address" => func!(invoke_workflow!("entry_address_workflow", "Entry", entry_address_workflow)),

                // Send a message directly to another node
                "hc_send" => func!(invoke_workflow!("send_workflow", "SendArgs", send_workflow)),

                // Allow a specified amount of time to pass
                "hc_sleep" => func!(invoke_workflow!("sleep_workflow", "nanos", sleep_workflow)),

                // Commit link deletion entry
                "hc_remove_link" => func!(invoke_workflow!("remove_link_wasm_workflow", "EntryWithHeader", remove_link_wasm_workflow)),

                //execute cryptographic function
                // "hc_crypto" => func!(invoke_workflow!("crypto_workflow", "CryptoArgs", crypto_workflow)),
                "hc_sign" => func!(invoke_workflow!("sign_workflow", "WasmString", sign_workflow)),
                "hc_decrypt" => func!(invoke_workflow!("decrypt_workflow", "WasmString", decrypt_workflow)),

                // Sign a block of data with a one-time key that is then shredded
                "hc_sign_one_time" => func!(invoke_workflow!("sign_one_time_workflow", "OneTimeSignArgs", sign_one_time_workflow)),

                // Verify that a block of data was signed by a given public key
                "hc_verify_signature" => func!(invoke_workflow!("verify_signature_workflow", "VerifySignatureArgs", verify_signature_workflow)),

                // Retrieve a list of identifiers of the secrets in the keystore
                "hc_keystore_list" => func!(invoke_workflow!("keystore_list_workflow", "()", keystore_list_workflow)),

                // Create a new random seed Secret in the keystore
                "hc_keystore_new_random" => func!(invoke_workflow!("keystore_new_random_workflow", "WasmString", keystore_new_random_workflow)),

                // Derive a new seed from an existing seed in the keystore
                "hc_keystore_derive_seed" => func!(invoke_workflow!("keystore_derive_seed_workflow", "WasmString", keystore_derive_seed_workflow)),

                // Create a new key (signing or encrypting) as derived from an existing seed in the keystore
                "hc_keystore_derive_key" => func!(invoke_workflow!("keystore_derive_key_workflow", "WasmString", keystore_derive_key_workflow)),

                // Sign a block of data using a key in the keystore
                "hc_keystore_sign" => func!(invoke_workflow!("keystore_sign_workflow", "WasmString", keystore_sign_workflow)),

                // Get the public key for a given secret
                "hc_keystore_get_public_key" => func!(invoke_workflow!("keystore_get_public_key_workflow", "WasmString", keystore_get_public_key_workflow)),

                // Commit a capability grant to the source chain
                "hc_commit_capability_grant" => func!(invoke_workflow!("commit_capability_grant_workflow", "CommitCapabilityGrantArgs", commit_capability_grant_workflow)),

                // Commit a capability grant to the source chain
                "hc_commit_capability_claim" => func!(invoke_workflow!("commit_capability_claim_workflow", "CommitCapabilityClaimArgs", commit_capability_claim_workflow)),

                // Send a DNA defined signal to UIs and other listeners
                "hc_emit_signal" => func!(invoke_workflow!("emit_signal_workflow", "EmitSignalArgs", emit_signal_workflow)),

                // send a meta
                "hc_meta" => func!(invoke_workflow!("meta_workflow", "MetaArgs", meta_workflow)),
            },
        };

        let new_instance = |wasm: &Vec<u8>| {
            Ok(instantiate(wasm, &wasm_imports).map_err(|e| HolochainError::from(e.to_string()))?)
        };

        let (context, zome_name) = if let WasmCallData::DirectCall(_, wasm) = self {
            return new_instance(&wasm);
        } else {
            match self {
                WasmCallData::ZomeCall(d) => (d.context.clone(), d.call.zome_name.clone()),
                WasmCallData::CallbackCall(d) => (d.context.clone(), d.call.zome_name.clone()),
                WasmCallData::DirectCall(_, _) => unreachable!(),
            }
        };

        let state_lock = context.state()?;
        // @TODO caching for wasm and/or modules, just reinstance them
        let wasm = state_lock
            .nucleus()
            .dna
            .as_ref()
            .unwrap()
            .zomes
            .get(&zome_name)
            .ok_or_else(|| HolochainError::new(&format!("No Ribosome found for Zome '{}'", zome_name)))?
            .code
            .code
            .clone();

        new_instance(&wasm)
    }
}

impl fmt::Display for WasmCallData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WasmCallData::ZomeCall(data) => write!(f, "ZomeCall({:?})", data.call),
            WasmCallData::CallbackCall(data) => write!(f, "CallbackCall({:?})", data.call),
            WasmCallData::DirectCall(name, _) => write!(f, "DirectCall({})", name),
        }
    }
}

impl fmt::Debug for WasmCallData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WasmCallData({})", self)
    }
}

/// Struct holding data of any call (callback or zome)
#[derive(Clone)]
pub struct CallData {
    pub context: Arc<Context>,
    pub zome_name: String,
    pub fn_name: String,
    pub parameters: JsonString,
}

/// Object holding data to pass around to invoked Zome API functions
// #[derive(Clone)]
pub struct Runtime {
    pub wasm_instance: Instance,

    /// data to be made available to the function at runtime
    pub data: WasmCallData,
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Runtime {
    pub fn zome_call_data(&self) -> Result<ZomeCallData, RuntimeError> {
        self.data.zome_call_data()
    }

    pub fn callback_call_data(&self) -> Result<CallbackCallData, RuntimeError> {
        self.data.callback_call_data()
    }

    pub fn call_data(&self) -> Result<CallData, RuntimeError> {
        self.data.call_data()
    }

    pub fn context(&self) -> Result<Arc<Context>, HolochainError> {
        self.data.context()
    }
}
