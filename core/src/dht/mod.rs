pub mod status;

use std;
use chain::pair::Pair;
use error::HolochainError;
use network::message::Message;
use network::message::put::Put;
use network::message::delete::Delete;
use network::message::modify::Modify;

// TODO IntoIterator trait
pub trait DHT {

    // state changes
    fn open (&mut self) -> Result<(), HolochainError>;
    fn close (&mut self) -> Result<(), HolochainError>;

    fn put (&mut self, message: Put) -> Result<(), HolochainError>;
    fn delete (&mut self, message: Delete) -> Result<(), HolochainError>;
    fn modify (&mut self, message: Modify) -> Result<(), HolochainError>;

    fn put_link (&mut self, message: Put) -> Result<(), HolochainError>;
    fn delete_link (&mut self, message: Delete) -> Result<(), HolochainError>;

    // traversal

    fn iter(&self) -> std::slice::Iter<Pair>;
    fn exists (&self, key: String, status_mask: status::StatusMask) -> Result<bool, HolochainError>;
    fn source (&self, key: String) -> Result<String, HolochainError>;
    fn get (&self, key: String) -> Result<Option<Pair>, HolochainError>;
    fn get_links (&self, key: String) -> Result<Option<Vec<Pair>>, HolochainError>;
    fn get_index (&self) -> Result<usize, HolochainError>;
    fn get_message <M: Message>(&self, index: usize) -> Result<M, HolochainError>;

    // serialization
    fn string (&self) -> Result<String, HolochainError>;
    fn json (&self) -> Result<String, HolochainError>;

}
