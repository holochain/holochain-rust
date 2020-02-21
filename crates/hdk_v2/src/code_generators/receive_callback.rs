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
                pub extern "C" fn receive(host_allocation_ptr: $crate::holochain_wasmer_guest::AllocationPtr) -> $crate::holochain_wasmer_guest::AllocationPtr {
                    let input: $crate::hdk::holochain_wasm_types::receive::ReceiveParams = $crate::holochain_wasmer_guest::host_args!(host_allocation_ptr);

                    fn execute(input: $crate::hdk::holochain_wasm_types::receive::ReceiveParams) -> String {
                        let #receive_from = input.from;
                        let #receive_param = input.payload;
                        #receive_blocks
                    }

                    $crate::holochain_wasmer_guest::ret!($crate::hdk::holochain_wasm_engine::holochain_json_string::json::RawString::from(execute(input)));
                }
            )*
        }
    }
}
