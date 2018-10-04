pub mod content;
pub mod eav;
pub mod file;
pub mod memory;
pub mod storage;

use self::{
    storage::ContentAddressableStorage,
    eav::EntityAttributeValueStorage,
    content::{Address, AddressableContent, Content},
    memory::MemoryStorage,
};
use error::HolochainError;
use std::collections::HashMap;

//pub trait RelatableContentStorage:
//    ContentAddressableStorage + EntityAttributeValueStorage;// {}
//
