pub mod message;

use chain::SourceChain;
use error::HolochainError;
use dht::message::put::Put;

pub trait DHT {

    fn genesis <'de, C: SourceChain<'de>>(&mut self, chain: &C) -> Result<(), HolochainError>;

    fn put <'de, C: SourceChain<'de>>(&mut self, chain: &C, message: Put, k: String) -> Result<(), HolochainError>;

}
