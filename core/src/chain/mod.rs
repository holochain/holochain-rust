// pub mod memory;
use error::HolochainError;
use hash_table::HashTable;
use hash_table::{entry::Entry, pair::Pair};

pub struct ChainIterator {

    table: Box<HashTable>,
    current: Option<Pair>,

}

impl ChainIterator {

    fn new<HT: HashTable> (table: HT, pair: &Option<Pair>) -> ChainIterator {
        ChainIterator{
            current: pair.clone(),
            table: table.box_clone(),
        }
    }

}

impl Iterator for ChainIterator {

    type Item = Pair;

    fn next(&mut self) -> Option<Pair> {
        self.current
        // @TODO should this be panicking?
        // let k = self.current.and_then(|p| Some(p.header().next()));
        // let n = self.table.get(&k).unwrap();
        // self.current = n;
        // self.current
    }

}

#[derive(Clone, Debug, PartialEq)]
pub struct Chain {

    table: Box<HashTable>,
    top: Option<Pair>,

}

impl Chain {

    pub fn new<HT: HashTable> (table: &HT) -> Chain {
        Chain{
            table: table.box_clone(),
            top: None,
        }
    }

    fn push<HT: HashTable> (&mut self, entry: &Entry) -> Result<Pair, HolochainError> {
        let pair = Pair::new(self, entry);

        if !(pair.validate()) {
            return Result::Err(HolochainError::new("attempted to push an invalid pair for this chain"))
        }

        let top_pair = self.top().and_then(|p| Some(p.key()));
        let next_pair = pair.header().next();

        if top_pair != next_pair {
            return Result::Err(HolochainError::new(
                &format!(
                    "top pair did not match next hash pair from pushed pair: {:?} vs. {:?}",
                    top_pair.clone(), next_pair.clone()
                )
            ))
        }

        // let mut validation_chain = self.clone();
        // validation_chain.top = Some(pair.clone());
        // validation_chain.pairs.insert(0, pair.clone());
        // if !validation_chain.validate() {
        //     return Result::Err(HolochainError::new("adding this pair would invalidate the source chain"))
        // }

        let result = self.table.commit(&pair);
        if result.is_ok() {
            self.top = Some(pair.clone());
        }
        match result {
            Result::Ok(_) => Result::Ok(pair),
            Result::Err(e) => Result::Err(e),
        }
    }

    pub fn table(&self) -> Box<HashTable> {
        self.table.clone()
    }

    // fn validate(&self) -> bool {
    //     self.pairs.iter().all(|p| p.validate())
    // }
    //
    pub fn iter(&self) -> ChainIterator {
        ChainIterator::new(self.table(), &self.top())
    }

    pub fn get<HT: HashTable> (&self, table: &HT, k: &str) -> Result<Option<Pair>, HolochainError> {
        table.get(k)
    }

    // fn get_entry (&self, table: &HT, entry_hash: &str) -> Option<Pair> {
    //     // @TODO - this is a slow way to do a lookup
    //     // @see https://github.com/holochain/holochain-rust/issues/50
    //     self
    //         .iter(table)
    //         .find(|p| p.entry().hash() == entry_hash)
    // }

    pub fn top(&self) -> Option<Pair> {
        self.top.clone()
    }

    pub fn top_type(&self, t: &str) -> Option<Pair> {
        self
            .iter()
            .find(|p| p.header().entry_type() == t)
    }

}

// pub trait SourceChain:
//     // IntoIterator +
//     Serialize {
//     /// append a pair to the source chain if the pair and new chain are both valid, else panic
//     fn push(&mut self, &Entry) -> Result<Pair, HolochainError>;
//
//     /// returns an iterator referencing pairs from top (most recent) to bottom (genesis)
//     fn iter(&self) -> std::slice::Iter<Pair>;
//
//     /// returns true if system and dApp validation is successful
//     fn validate(&self) -> bool;
//
//     /// returns a pair for a given header hash
//     fn get(&self, k: &str) -> Option<Pair>;
//
//     /// returns a pair for a given entry hash
//     fn get_entry(&self, k: &str) -> Option<Pair>;
//
//     /// returns the top (most recent) pair from the source chain
//     fn top(&self) -> Option<Pair>;
//
//     /// returns the top (most recent) pair of a given type from the source chain
//     fn top_type(&self, t: &str) -> Option<Pair>;
// }

#[cfg(test)]
pub mod tests {

    use super::Chain;
    use hash_table::memory::MemTable;
    use hash_table::memory::tests::test_table;

    pub fn test_chain() -> Chain<MemTable> {
        Chain::new(test_table())
    }

    #[test]
    fn new() {
        test_chain();
    }

}
