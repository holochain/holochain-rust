pub mod message;

use chain::SourceChain;
use error::HolochainError;
use message::put::PutMessage;

pub trait DHT {

    fn genesis <'de, C: SourceChain<'de>>(&mut self, chain: &C) -> Result<(), HolochainError>;

    fn put <'de, C: SourceChain<'de>>(&mut self, chain: &C, message: PutMessage, k: String) -> Result<(), HolochainError>;

}
