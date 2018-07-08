use holochain_core::chain::pair::Pair;
use holochain_core::chain::SourceChain;

pub fn test_pair() -> Pair {
    let mut c = super::test_chain();
    c.push(&::chain::entry::test_entry())
}
