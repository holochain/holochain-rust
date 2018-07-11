pub mod status;

use error::HolochainError;
use chain::pair::Pair;

pub trait HashTable {

    // state changes
    fn open (&mut self) -> Result<(), HolochainError>;
    fn close (&mut self) -> Result<(), HolochainError>;

    fn put (&mut self, pair: &Pair) -> Result<(), HolochainError>;
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;
    fn get (&self, key: String) -> Result<Option<Pair>, HolochainError>;
    fn modify (&mut self, old_pair: &Pair, new_pair: &Pair) -> Result<(), HolochainError>;
    fn retract (&mut self, pair: &Pair) -> Result<(), HolochainError>;

}
