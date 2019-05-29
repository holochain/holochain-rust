use crate::zome_code_def::ZomeCodeDef;
use proc_macro2::TokenStream;
use quote::quote;

impl ZomeCodeDef {
    pub fn receive_callback(&self) -> TokenStream {
        let (receive_blocks, receive_from, receive_param) = match &self.receive_callback {
            None => (Vec::new(), Vec::new(), Vec::new()),
            Some(callback) => (
                vec![callback.code.clone()],
                vec![callback.from_param.clone()],
                vec![callback.message_param.clone()],
            ),
        };

        quote! {
            #(
                #[no_mangle]
                pub extern "C" fn receive(encoded_allocation_of_input: hdk::holochain_core_types::error::RibosomeEncodingBits) -> hdk::holochain_core_types::error::RibosomeEncodingBits {
                    let maybe_allocation = hdk::holochain_wasm_utils::memory::allocation::WasmAllocation::try_from_ribosome_encoding(encoded_allocation_of_input);
                    let allocation = match maybe_allocation {
                        Ok(allocation) => allocation,
                        Err(allocation_error) => return hdk::holochain_core_types::error::RibosomeEncodedValue::from(allocation_error).into(),
                    };
                    let init = hdk::global_fns::init_global_memory(allocation);
                    if init.is_err() {
                        return hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                            init
                        ).into();
                    }

                    // Deserialize input
                    let input = load_json!(encoded_allocation_of_input);

                    fn execute(input: hdk::holochain_wasm_utils::api_serialization::receive::ReceiveParams) -> String {
                        let #receive_from = input.from;
                        let #receive_param = input.payload;
                        #receive_blocks
                    }

                    hdk::holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result(
                        hdk::global_fns::write_json(
                            JsonString::from_json(&execute(input))
                        )
                    ).into()
                }
            )*
        }
    }
}
