use eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value};
use error::HolochainError;
use std::collections::HashSet;

pub struct EavFileStorage {}

impl EavFileStorage {
    pub fn new() -> EavFileStorage {
        EavFileStorage {}
    }
}

impl EntityAttributeValueStorage for EavFileStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        unimplemented!("unimplemented for add_eav")
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        unimplemented!("unimplemented fetch for fetch_eav")
    }
}
