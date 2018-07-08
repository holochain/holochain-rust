pub mod message;
pub mod status;

use chain::SourceChain;
use error::HolochainError;
use dht::message::put::Put;
use dht::message::delete::Delete;
use dht::message::modify::Modify;

pub trait DHT {

    // setup/teardown
    fn genesis (&mut self) -> Result<(), HolochainError>;
    fn open (&mut self) -> Result<(), HolochainError>;
    fn close (&mut self) -> Result<(), HolochainError>;

    // messages

    fn put (&mut self, message: Put) -> Result<(), HolochainError>;
    fn delete (&mut self, message: Delete) -> Result<(), HolochainError>;
    fn modify (&mut self, message: Modify) -> Result<(), HolochainError>;

    // queries

    fn exists (&self, key: String, status_mask: status::StatusMask) -> Result<bool, HolochainError>;
    fn source (&self, key: String) -> Result<String, HolochainError>;
    fn get (&self, key: String) -> Result<Option<Pair>

}
