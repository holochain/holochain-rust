use crate::zome_code_def::ZomeCodeDef;
use proc_macro2::TokenStream;
use quote::quote;

impl ZomeCodeDef {
    pub fn init(&self) -> TokenStream {
        let init = &self.init;

        quote! {
            #[no_mangle]
            pub extern "C" fn init(host_allocation_ptr: hdk::holochain_core_types::error::AllocationPtr) -> hdk::holochain_core_types::error::AllocationPtr {
                let maybe_allocation = hdk::holochain_wasm_utils::memory::allocation::WasmAllocation::try_from_ribosome_encoding(host_allocation_ptr);
                let allocation = match maybe_allocation {
                    Ok(allocation) => allocation,
                    Err(allocation_error) => return hdk::holochain_core_types::error::RibosomeReturnValue::from(allocation_error).into(),
                };
                let init = hdk::global_fns::init_global_memory(allocation);
                if init.is_err() {
                    return hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                        init
                    ).into();
                }

                fn execute() -> Result<(), String> {
                    #init
                }

                match execute() {
                    Ok(_) => hdk::holochain_core_types::error::RibosomeReturnValue::Success.into(),
                    Err(e) => hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                        hdk::global_fns::write_json(
                            hdk::holochain_wasm_utils::holochain_json_api::json::RawString::from(e)
                        )
                    ).into(),
                }
            }
        }
    }
}
