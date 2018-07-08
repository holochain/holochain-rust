pub mod message;

use chain::SourceChain;
use error::HolochainError;
use dht::message::put::Put;
use dht::message::delete::Delete;
use dht::message::modify::Modify;

pub trait DHT {

    fn genesis <'de, C: SourceChain<'de>>(&mut self, chain: &C) -> Result<(), HolochainError>;

    fn put <'de, C: SourceChain<'de>>(&mut self, chain: &C, message: Put) -> Result<(), HolochainError>;

    fn delete (&mut self, message: Delete) -> Result<(), HolochainError>;

    fn modify (&mut self, message: Modify) -> Result<(), HolochainError>;

}
