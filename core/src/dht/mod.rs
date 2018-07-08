pub mod message;

use chain::SourceChain;
use error::HolochainError;
use dht::message::put::Put;
use dht::message::delete::Delete;
use dht::message::modify::Modify;

pub trait DHT {

    fn genesis <'de, C: SourceChain<'de>>(&mut self, chain: &C) -> Result<(), HolochainError>;

    fn put <'de, C: SourceChain<'de>>(&mut self, chain: &C, message: Put, k: String) -> Result<(), HolochainError>;

    fn delete (&mut self, message: Delete, k: String) -> Result<(), HolochainError>;

    fn modify (&mut self, message: Modify, k_old: String, k_new: String) -> Result<(), HolochainError>;

}
