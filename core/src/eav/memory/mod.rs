use eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value};
use error::HolochainError;
use std::collections::HashSet;

pub struct EavMemoryStorage {
    eavs: HashSet<EntityAttributeValue>,
}

impl EavMemoryStorage {
    pub fn new() -> EavMemoryStorage {
        EavMemoryStorage {
            eavs: HashSet::new(),
        }
    }
}

impl EntityAttributeValueStorage for EavMemoryStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        self.eavs.insert(eav.clone());
        Ok(())
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        Ok(self
            .eavs
            .iter()
            .cloned()
            .filter(|e| EntityAttributeValue::filter_on_eav::<Entity>(e.entity(), &entity))
            .filter(|e| EntityAttributeValue::filter_on_eav::<Attribute>(e.attribute(), &attribute))
            .filter(|e| EntityAttributeValue::filter_on_eav::<Value>(e.value(), &value))
            .collect::<HashSet<EntityAttributeValue>>())
    }
}

#[cfg(test)]
pub mod tests {
    use cas::content::{tests::ExampleAddressableContent, AddressableContent};
    use eav::{memory::EavMemoryStorage, EntityAttributeValue, EntityAttributeValueStorage};
    use std::collections::HashSet;

    #[test]
    fn memory_eav_round_trip() {
        let entity_content = ExampleAddressableContent::from_content(&"foo".to_string());
        let attribute = "favourite-color".to_string();
        let value_content = ExampleAddressableContent::from_content(&"blue".to_string());
        let eav = EntityAttributeValue::new(
            &entity_content.address(),
            &attribute,
            &value_content.address(),
        );
        let mut eav_storage = EavMemoryStorage::new();

        assert_eq!(
            HashSet::new(),
            eav_storage
                .fetch_eav(
                    Some(entity_content.address()),
                    Some(attribute.clone()),
                    Some(value_content.address())
                )
                .expect("could not fetch eav"),
        );

        eav_storage.add_eav(&eav).expect("could not add eav");

        let mut expected = HashSet::new();
        expected.insert(eav.clone());
        // some examples of constraints that should all return the eav
        for (e, a, v) in vec![
            // constrain all
            (
                Some(entity_content.address()),
                Some(attribute.clone()),
                Some(value_content.address()),
            ),
            // open entity
            (None, Some(attribute.clone()), Some(value_content.address())),
            // open attribute
            (
                Some(entity_content.address()),
                None,
                Some(value_content.address()),
            ),
            // open value
            (
                Some(entity_content.address()),
                Some(attribute.clone()),
                None,
            ),
            // open
            (None, None, None),
        ] {
            assert_eq!(
                expected,
                eav_storage.fetch_eav(e, a, v).expect("could not fetch eav"),
            );
        }
    }

    #[test]
    fn memory_eav_one_to_many() {
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "one_to_many".to_string();

        let mut eav_storage = EavMemoryStorage::new();
        let mut expected = HashSet::new();
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let eav = EntityAttributeValue::new(&one.address(), &attribute, &many.address());
            eav_storage.add_eav(&eav).expect("could not add eav");
            expected.insert(eav);
        }

        // throw an extra thing referencing many to show fetch ignores it
        let two = ExampleAddressableContent::from_content(&"foo".to_string());
        for many in vec![many_one.clone(), many_three.clone()] {
            eav_storage
                .add_eav(&EntityAttributeValue::new(
                    &two.address(),
                    &attribute,
                    &many.address(),
                ))
                .expect("could not add eav");
        }

        // show the many results for one
        assert_eq!(
            expected,
            eav_storage
                .fetch_eav(Some(one.address()), Some(attribute.clone()), None)
                .expect("could not fetch eav"),
        );

        // show one for the many results
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let mut expected_one = HashSet::new();
            expected_one.insert(EntityAttributeValue::new(
                &one.address(),
                &attribute.clone(),
                &many.address(),
            ));
            assert_eq!(
                expected_one,
                eav_storage
                    .fetch_eav(None, Some(attribute.clone()), Some(many.address()))
                    .expect("could not fetch eav"),
            );
        }
    }

    #[test]
    fn memory_eav_many_to_one() {
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "many_to_one".to_string();

        let mut eav_storage = EavMemoryStorage::new();
        let mut expected = HashSet::new();
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let eav = EntityAttributeValue::new(&many.address(), &attribute, &one.address());
            eav_storage.add_eav(&eav).expect("could not add eav");
            expected.insert(eav);
        }

        // throw an extra thing referenced by many to show fetch ignores it
        let two = ExampleAddressableContent::from_content(&"foo".to_string());
        for many in vec![many_one.clone(), many_three.clone()] {
            eav_storage
                .add_eav(&EntityAttributeValue::new(
                    &many.address(),
                    &attribute,
                    &two.address(),
                ))
                .expect("could not add eav");
        }

        // show the many referencing one
        assert_eq!(
            expected,
            eav_storage
                .fetch_eav(None, Some(attribute.clone()), Some(one.address()))
                .expect("could not fetch eav"),
        );

        // show one for the many results
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let mut expected_one = HashSet::new();
            expected_one.insert(EntityAttributeValue::new(
                &many.address(),
                &attribute.clone(),
                &one.address(),
            ));
            assert_eq!(
                expected_one,
                eav_storage
                    .fetch_eav(Some(many.address()), Some(attribute.clone()), None)
                    .expect("could not fetch eav"),
            );
        }
    }

}
