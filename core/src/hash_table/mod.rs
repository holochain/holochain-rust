pub mod status;
pub mod entry;
pub mod header;
pub mod pair;
pub mod memory;

use error::HolochainError;
use hash_table::pair::Pair;

pub trait HashTable {

    // state changes
    fn open (&mut self) -> Result<(), HolochainError>;
    fn close (&mut self) -> Result<(), HolochainError>;

    // crud
    fn commit (&mut self, pair: &Pair) -> Result<(), HolochainError>;
    fn get (&self, key: &str) -> Result<Option<Pair>, HolochainError>;
    fn modify (&mut self, old_pair: &Pair, new_pair: &Pair) -> Result<(), HolochainError>;
    fn retract (&mut self, pair: &Pair) -> Result<(), HolochainError>;

    // query
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;

}
