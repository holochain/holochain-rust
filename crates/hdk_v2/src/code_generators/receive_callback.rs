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
                pub extern "C" fn receive(host_allocation_ptr: holochain_wasmer_guest::AllocationPtr) -> holochain_wasmer_guest::AllocationPtr {
                    let input = holochain_wasmer_guest::host_args!(host_allocation_ptr, hdk::holochain_wasm_utils::api_serialization::receive::ReceiveParams);

                    fn execute(input: hdk::holochain_wasm_utils::api_serialization::receive::ReceiveParams) -> String {
                        let #receive_from = input.from;
                        let #receive_param = input.payload;
                        #receive_blocks
                    }

                    ret!(WasmResult::Ok(JsonString::from_json(&execute(input)));
                }
            )*
        }
    }
}
