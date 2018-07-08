pub mod pair;
pub mod entry;
use holochain_core::chain::memory::MemChain;

pub fn test_chain () -> MemChain {
    MemChain::new()
}
