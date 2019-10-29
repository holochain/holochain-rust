pub mod cas;
pub mod fixture;

use rusoto_dynamodb::{AttributeDefinition, AttributeValue, KeySchemaElement};

pub type TableName = String;

pub fn hash_key(attribute_name: &str) -> KeySchemaElement {
    KeySchemaElement {
        attribute_name: attribute_name.into(),
        key_type: "HASH".into(),
    }
}

pub fn string_attribute_definition(attribute_name: &str) -> AttributeDefinition {
    AttributeDefinition {
        attribute_name: attribute_name.into(),
        attribute_type: "S".into(),
    }
}

pub fn string_attribute_value(value: &str) -> AttributeValue {
    AttributeValue {
        s: Some(value.to_string()),
        ..Default::default()
    }
}

pub fn bool_attribute_value(value: bool) -> AttributeValue {
    AttributeValue {
        bool: Some(value),
        ..Default::default()
    }
}

pub fn blob_attribute_value(value: &Vec<u8>) -> AttributeValue {
    AttributeValue {
        b: Some(value.as_slice().into()),
        ..Default::default()
    }
}

pub fn number_attribute_value(value: u64) -> AttributeValue {
    AttributeValue {
        n: Some(value.to_string()),
        ..Default::default()
    }
}

pub fn string_set_attribute_value(value: Vec<String>) -> AttributeValue {
    AttributeValue {
        ss: Some(value),
        ..Default::default()
    }
}

pub fn list_attribute_value(value: Vec<AttributeValue>) -> AttributeValue {
    AttributeValue {
        l: Some(value),
        ..Default::default()
    }
}

#[cfg(test)]
pub mod test {

    use crate::dht::bbdht::dynamodb::schema::{
        fixture::attribute_name_fresh, hash_key, string_attribute_definition,
        string_attribute_value,
    };
    use rusoto_dynamodb::{AttributeDefinition, AttributeValue, KeySchemaElement};

    #[test]
    fn hash_key_test() {
        let attribute_name = attribute_name_fresh();

        let result = hash_key(&attribute_name);

        assert_eq!(
            KeySchemaElement {
                attribute_name: attribute_name.to_string(),
                key_type: String::from("HASH"),
            },
            result,
        );
    }

    #[test]
    fn string_attribute_definition_test() {
        let attribute_name = attribute_name_fresh();

        let result = string_attribute_definition(&attribute_name);

        assert_eq!(
            AttributeDefinition {
                attribute_name: attribute_name.into(),
                attribute_type: "S".into(),
            },
            result,
        );
    }

    #[test]
    fn string_attribute_value_test() {
        let value = String::from("foo");

        let result = string_attribute_value(&value);

        assert_eq!(
            AttributeValue {
                b: None,
                bool: None,
                bs: None,
                l: None,
                m: None,
                n: None,
                ns: None,
                null: None,
                s: Some(value.clone()),
                ss: None,
            },
            result,
        );
    }

}
