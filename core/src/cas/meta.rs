use error::HolochainError;
use cas::content::Address;

/// Meta represents an extended form of EAV (entity-attribute-value) data
/// implemented on top of cas::storage::ContentAddressableStorage
/// @see https://en.wikipedia.org/wiki/Entity%E2%80%93attribute%E2%80%93value_model
/// Address of AddressableContent representing the EAV entity
type Entity = Address;
/// schema of all possible EAV attributes
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Attribute {
    CrudStatus,
    CrudLink,
}

/// Address of AddressableContent representing the EAV value
type Value = Address;

// @TODO do we need this?
// unique (local to the source) monotonically increasing number that can be used for crdt/ordering
// @see https://papers.radixdlt.com/tempo/#logical-clocks
// type Index ...

// @TODO do we need this?
// source agent asserting the meta
// type Source ...

pub struct Meta {
    entity: Entity,
    attribute: Attribute,
    value: Value,
    // index: Index,
    // source: Source,
}

impl Meta {
    pub fn entity(&self) -> Entity {
        self.entity.clone()
    }

    pub fn attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }
}

/// meta storage
pub trait MetaStorage {
    fn add_meta(&mut self, meta: Meta) -> Result<(), HolochainError>;
    fn fetch_meta_ea(&self, entity: Option<Entity>, attribute: Option<Attribute>) -> Result<Vec<Meta>, HolochainError>;
}

#[cfg(test)]
pub mod tests {
    use error::HolochainError;
    use std::collections::HashMap;
    use cas::meta::Entity;
    use cas::meta::Attribute;
    use cas::meta::MetaStorage;
    use cas::meta::Meta;

    pub struct ExampleMetaStorage {
        index_eav: HashMap<(Entity, Attribute), Vec<Meta>>,
    }

    impl MetaStorage for ExampleMetaStorage {
        fn add_meta(&mut self, meta: Meta) -> Result<(), HolochainError> {
            let idx = (meta.entity(), meta.attribute());
            let mut metas = self.index_eav.get_mut(&idx);
            match metas {
                Some(v) => v.push(meta),
                None => {self.index_eav.insert(idx, vec![meta]);},
            };
            Ok(())
        }

        fn fetch_meta_ea(&self, entity: Option<Entity>, attribute: Option<Attribute>) -> Result<Vec<Meta>, HolochainError> {
            Ok(
            match entity {
                Some(e) => {
                    match attribute {
                        Some(a) => self.index_eav.get(&(e, a)),
                        None => self.fetch_meta_e(e),
                    }
                },
                None => {
                    match attribute {
                        Some(_) => unreachable!(),
                        None => {
                            self.fetch_meta_all()
                        }
                    }
                }
            })
        }
    }

    #[test]
    fn example_meta_round_trip () {

    }
}
