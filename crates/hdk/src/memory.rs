use holochain_wasm_utils::memory::handler::WasmMemoryHandler;
use holochain_wasm_utils::memory::Top;

#[derive(Default)]
pub struct WasmMemory {}

impl WasmMemoryHandler for WasmMemory {

    fn set_top(&self, top: Top) -> Top {
       let bytes = Top::to_le_bytes(top);
   }
}
