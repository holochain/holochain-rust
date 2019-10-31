use crate::{
    dht::bbdht::{
        dynamodb::{
            api::item::write::should_put_item_retry,
            client::Client,
            schema::{
                blob_attribute_value,
                cas::{
                    ADDRESS_KEY, ASPECT_ADDRESS_KEY, ASPECT_KEY, ASPECT_LIST_KEY,
                    ASPECT_PUBLISH_TS_KEY, ASPECT_TYPE_HINT_KEY,
                },
                number_attribute_value, string_attribute_value, string_set_attribute_value,
                TableName,
            },
        },
        error::BbDhtResult,
    },
    trace::{tracer, LogContext},
};
use holochain_persistence_api::cas::content::Address;
use lib3h_protocol::data_types::EntryAspectData;
use rusoto_dynamodb::{AttributeValue, DynamoDb, PutItemInput, UpdateItemInput};
use std::collections::HashMap;

pub fn aspect_list_to_attribute(aspect_list: &Vec<EntryAspectData>) -> AttributeValue {
    string_set_attribute_value(
        aspect_list
            .iter()
            .map(|aspect| aspect.aspect_address.to_string())
            .collect(),
    )
}

pub fn put_aspect(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    aspect: &EntryAspectData,
) -> BbDhtResult<()> {
    tracer(&log_context, "put_aspect");

    let mut aspect_item = HashMap::new();
    aspect_item.insert(
        String::from(ADDRESS_KEY),
        string_attribute_value(&aspect.aspect_address.to_string()),
    );

    aspect_item.insert(
        String::from(ASPECT_ADDRESS_KEY),
        string_attribute_value(&aspect.aspect_address.to_string()),
    );

    aspect_item.insert(
        String::from(ASPECT_TYPE_HINT_KEY),
        string_attribute_value(&aspect.type_hint),
    );

    aspect_item.insert(
        String::from(ASPECT_KEY),
        blob_attribute_value(&aspect.aspect),
    );

    aspect_item.insert(
        String::from(ASPECT_PUBLISH_TS_KEY),
        number_attribute_value(aspect.publish_ts),
    );

    if should_put_item_retry(
        log_context,
        client
            .put_item(PutItemInput {
                table_name: table_name.to_string(),
                item: aspect_item,
                ..Default::default()
            })
            .sync(),
    )? {
        put_aspect(log_context, client, table_name, aspect)
    } else {
        Ok(())
    }
}

pub fn append_aspect_list_to_entry(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    entry_address: &Address,
    aspect_list: &Vec<EntryAspectData>,
) -> BbDhtResult<()> {
    tracer(&log_context, "append_aspects");

    // need to append all the aspects before making them discoverable under the entry
    for aspect in aspect_list {
        put_aspect(&log_context, &client, &table_name, &aspect)?;
    }

    // the aspect addressses live under the entry address
    let mut aspect_addresses_key = HashMap::new();
    aspect_addresses_key.insert(
        String::from(ADDRESS_KEY),
        string_attribute_value(&String::from(entry_address.to_owned())),
    );

    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(
        ":aspects".to_string(),
        aspect_list_to_attribute(&aspect_list),
    );

    let mut expression_attribute_names = HashMap::new();
    expression_attribute_names.insert("#aspect_list".to_string(), ASPECT_LIST_KEY.to_string());

    // https://stackoverflow.com/questions/31288085/how-to-append-a-value-to-list-attribute-on-aws-dynamodb
    let update_expression = "ADD #aspect_list :aspects";

    let aspect_list_update = UpdateItemInput {
        table_name: table_name.to_string(),
        key: aspect_addresses_key,
        update_expression: Some(update_expression.to_string()),
        expression_attribute_names: Some(expression_attribute_names),
        expression_attribute_values: Some(expression_attribute_values),
        ..Default::default()
    };

    client.update_item(aspect_list_update).sync()?;
    Ok(())
}

#[cfg(test)]
pub mod tests {

    use crate::{
        aspect::fixture::{aspect_list_fresh, entry_aspect_data_fresh},
        dht::bbdht::dynamodb::{
            api::{
                aspect::write::{
                    append_aspect_list_to_entry, aspect_list_to_attribute, put_aspect,
                },
                item::read::get_item_by_address,
                table::{create::ensure_cas_table, exist::table_exists, fixture::table_name_fresh},
            },
            client::local::local_client,
            schema::{
                cas::{ADDRESS_KEY, ASPECT_LIST_KEY},
                string_attribute_value,
            },
        },
        entry::fixture::entry_hash_fresh,
        trace::tracer,
    };
    use std::collections::HashMap;

    #[test]
    fn put_aspect_test() {
        let log_context = "put_aspect_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let entry_aspect = entry_aspect_data_fresh();

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // cas exists
        assert!(table_exists(&log_context, &local_client, &table_name).is_ok());

        // put aspect
        assert!(put_aspect(&log_context, &local_client, &table_name, &entry_aspect).is_ok());
    }

    #[test]
    fn append_aspects_to_entry_test() {
        let log_context = "append_aspects_to_entry_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let entry_address = entry_hash_fresh();
        let aspect_list = aspect_list_fresh();

        let mut expected = HashMap::new();
        expected.insert(
            ASPECT_LIST_KEY.to_string(),
            aspect_list_to_attribute(&aspect_list),
        );
        expected.insert(
            ADDRESS_KEY.to_string(),
            string_attribute_value(&String::from(entry_address.clone())),
        );

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // cas exists
        assert!(table_exists(&log_context, &local_client, &table_name).is_ok());

        // trash/idempotency loop
        for _ in 0..3 {
            // append aspects
            assert!(append_aspect_list_to_entry(
                &log_context,
                &local_client,
                &table_name,
                &entry_address,
                &aspect_list
            )
            .is_ok());

            // get matches
            match get_item_by_address(&log_context, &local_client, &table_name, &entry_address) {
                Ok(get_item_output) => match get_item_output {
                    Some(item) => {
                        assert_eq!(expected["address"], item["address"],);
                        assert_eq!(
                            expected["aspect_list"].ss.iter().count(),
                            item["aspect_list"].ss.iter().count(),
                        );
                    }
                    None => {
                        tracer(&log_context, "get matches None");
                        panic!("None");
                    }
                },
                Err(err) => {
                    tracer(&log_context, "get matches err");
                    panic!("{:?}", err);
                }
            }
        }
    }
}
