use crate::dht::bbdht::dynamodb::schema::{hash_key, string_attribute_definition};
use holochain_persistence_api::cas::content::Address;
use rusoto_dynamodb::{AttributeDefinition, KeySchemaElement};

pub const ADDRESS_KEY: &str = "address";
pub const CONTENT_KEY: &str = "content";
pub const ASPECT_LIST_KEY: &str = "aspect_list";
pub const ASPECT_ADDRESS_KEY: &str = "aspect_address";
pub const ASPECT_TYPE_HINT_KEY: &str = "aspect_type_hint";
pub const ASPECT_KEY: &str = "aspect";
pub const ASPECT_PUBLISH_TS_KEY: &str = "aspect_publish_ts";

// direct messaging keys
pub const INBOX_KEY_PREFIX: &str = "inbox_";
pub const REQUEST_IDS_KEY: &str = "request_ids";
pub const REQUEST_IDS_SEEN_KEY: &str = "request_ids_seen";
pub const MESSAGE_SPACE_ADDRESS_KEY: &str = "message_space_address";
pub const MESSAGE_FROM_KEY: &str = "message_from";
pub const MESSAGE_TO_KEY: &str = "message_to";
pub const MESSAGE_CONTENT_KEY: &str = "message_content";
pub const MESSAGE_IS_RESPONSE_KEY: &str = "message_is_response";

pub fn inbox_key(agent_id: &Address) -> String {
    format!("{}{}", INBOX_KEY_PREFIX, agent_id)
}

pub fn address_key_schema() -> KeySchemaElement {
    hash_key(ADDRESS_KEY)
}

pub fn content_key_schema() -> KeySchemaElement {
    hash_key(CONTENT_KEY)
}

pub fn key_schema_cas() -> Vec<KeySchemaElement> {
    vec![address_key_schema()]
}

pub fn address_attribute_definition() -> AttributeDefinition {
    string_attribute_definition(ADDRESS_KEY)
}

pub fn content_attribute_definition() -> AttributeDefinition {
    string_attribute_definition(CONTENT_KEY)
}

pub fn attribute_definitions_cas() -> Vec<AttributeDefinition> {
    vec![
        address_attribute_definition(),
        // content_attribute_definition(),
    ]
}

#[cfg(test)]
pub mod tests {

    use crate::{
        dht::bbdht::dynamodb::schema::cas::{
            address_attribute_definition, address_key_schema, attribute_definitions_cas,
            content_attribute_definition, content_key_schema, key_schema_cas, ADDRESS_KEY,
            CONTENT_KEY,
        },
        trace::tracer,
    };
    use rusoto_dynamodb::{AttributeDefinition, KeySchemaElement};

    #[test]
    fn address_key_schema_test() {
        let log_context = "address_key_schema_test";

        tracer(&log_context, "compare values");
        assert_eq!(
            KeySchemaElement {
                attribute_name: ADDRESS_KEY.to_string(),
                key_type: "HASH".into(),
            },
            address_key_schema(),
        );
    }

    #[test]
    fn content_key_schema_test() {
        let log_context = "context_key_schema_test";

        tracer(&log_context, "compare values");
        assert_eq!(
            KeySchemaElement {
                attribute_name: CONTENT_KEY.to_string(),
                key_type: "HASH".into(),
            },
            content_key_schema(),
        );
    }

    #[test]
    fn key_schema_cas_test() {
        let log_context = "key_schema_cas_test";

        tracer(&log_context, "compare values");
        assert_eq!(
            vec![KeySchemaElement {
                attribute_name: ADDRESS_KEY.to_string(),
                key_type: "HASH".into(),
            }],
            key_schema_cas()
        );
    }

    #[test]
    fn address_attribute_definition_test() {
        let log_context = "address_attribute_definition_test";

        tracer(&log_context, "compare values");
        assert_eq!(
            AttributeDefinition {
                attribute_name: ADDRESS_KEY.to_string(),
                attribute_type: "S".into(),
            },
            address_attribute_definition(),
        );
    }

    #[test]
    fn content_attribute_definition_test() {
        let log_context = "content_attribute_definition_test";

        tracer(&log_context, "compare values");
        assert_eq!(
            AttributeDefinition {
                attribute_name: CONTENT_KEY.to_string(),
                attribute_type: "S".into(),
            },
            content_attribute_definition(),
        );
    }

    #[test]
    fn attribute_definitions_cas_test() {
        let log_context = "attribute_definitions_cas_test";

        tracer(&log_context, "compare values");
        assert_eq!(
            address_attribute_definition(),
            attribute_definitions_cas()[0]
        );
    }
}
