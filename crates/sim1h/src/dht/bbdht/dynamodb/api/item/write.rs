use crate::dht::bbdht::dynamodb::api::item::Item;
use crate::dht::bbdht::dynamodb::client::Client;
use crate::dht::bbdht::dynamodb::schema::cas::ADDRESS_KEY;
use crate::dht::bbdht::dynamodb::schema::cas::CONTENT_KEY;
use crate::dht::bbdht::dynamodb::schema::string_attribute_value;
use crate::dht::bbdht::dynamodb::schema::TableName;
use crate::dht::bbdht::error::BbDhtResult;
use crate::trace::tracer;
use crate::trace::LogContext;
use holochain_persistence_api::cas::content::AddressableContent;
use rusoto_core::RusotoError;
use rusoto_dynamodb::DynamoDb;
use rusoto_dynamodb::PutItemError;
use rusoto_dynamodb::PutItemInput;
use rusoto_dynamodb::PutItemOutput;
use std::collections::HashMap;

pub fn content_to_item(content: &dyn AddressableContent) -> Item {
    let mut item = HashMap::new();
    item.insert(
        String::from(ADDRESS_KEY),
        string_attribute_value(&String::from(content.address())),
    );
    item.insert(
        String::from(CONTENT_KEY),
        string_attribute_value(&String::from(content.content())),
    );
    item
}

pub fn should_put_item_retry(
    log_context: &LogContext,
    put_item_result: Result<PutItemOutput, RusotoError<PutItemError>>,
) -> BbDhtResult<bool> {
    match put_item_result {
        // no need to retry any success
        Ok(_) => Ok(false),
        Err(RusotoError::Service(err)) => match err {
            PutItemError::InternalServerError(err) => {
                // retry InternalServerErrors as these often seem to be temporary
                tracer(
                    &log_context,
                    &format!("retry Service InternalServerError {:?}", err),
                );
                Ok(true)
            }
            PutItemError::ProvisionedThroughputExceeded(err) => {
                // retry throughput issues as these will hopefully recover
                tracer(
                    &log_context,
                    &format!("retry Service ProvisionedThroughputExceeded {:?}", err),
                );
                Ok(true)
            }
            PutItemError::RequestLimitExceeded(err) => {
                // retry request limits as these will hopefully recover
                tracer(
                    &log_context,
                    &format!("retry put_aspect Service RequestLimitExceeded {:?}", err),
                );
                Ok(true)
            }
            PutItemError::TransactionConflict(err) => {
                // retry transaction conflicts
                tracer(
                    &log_context,
                    &format!("retry Service TransactionConflict {:?}", err),
                );
                Ok(true)
            }
            // forward other put item errors back up the stack
            _ => Err(err.into()),
        },
        Err(RusotoError::Unknown(err)) => {
            // retry anything we don't know about
            tracer(&log_context, &format!("retry Unknown {:?}", err));
            Ok(true)
        }
        // forward other errors back up the stack
        Err(err) => Err(err.into()),
    }
}

pub fn ensure_content(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    content: &dyn AddressableContent,
) -> BbDhtResult<()> {
    tracer(&log_context, "ensure_content");

    if should_put_item_retry(
        log_context,
        client
            .put_item(PutItemInput {
                item: content_to_item(content),
                table_name: table_name.to_string(),
                ..Default::default()
            })
            .sync(),
    )? {
        ensure_content(log_context, client, table_name, content)
    } else {
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

    use crate::dht::bbdht::dynamodb::api::item::fixture::content_fresh;
    use crate::dht::bbdht::dynamodb::api::item::write::content_to_item;
    use crate::dht::bbdht::dynamodb::api::item::write::ensure_content;
    use crate::dht::bbdht::dynamodb::api::table::create::ensure_cas_table;
    use crate::dht::bbdht::dynamodb::api::table::exist::table_exists;
    use crate::dht::bbdht::dynamodb::api::table::fixture::table_name_fresh;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::trace::tracer;
    use rusoto_dynamodb::DynamoDb;
    use rusoto_dynamodb::Put;
    use rusoto_dynamodb::TransactWriteItem;
    use rusoto_dynamodb::TransactWriteItemsInput;

    #[test]
    /// older versions of dynamodb don't support transact writes
    /// test that this version is supported
    fn transact_write_item_test() {
        let log_context = "transact_write_item_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let content_a = content_fresh();
        let content_b = content_fresh();

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // cas exists
        assert!(table_exists(&log_context, &local_client, &table_name)
            .expect("could not check table exists"));

        // transact
        local_client
            .transact_write_items(TransactWriteItemsInput {
                transact_items: vec![
                    TransactWriteItem {
                        put: Some(Put {
                            table_name: table_name.clone(),
                            item: content_to_item(&content_a),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    TransactWriteItem {
                        put: Some(Put {
                            table_name: table_name.clone(),
                            item: content_to_item(&content_b),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })
            .sync()
            .expect("could not transact write items");
    }

    #[test]
    fn ensure_content_test() {
        let log_context = "ensure_content_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let content = content_fresh();

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // cas exists
        assert!(table_exists(&log_context, &local_client, &table_name)
            .expect("could not check table exists"));

        // ensure content
        assert!(ensure_content(&log_context, &local_client, &table_name, &content).is_ok());

        // thrash a bit
        for _ in 0..100 {
            assert!(ensure_content(&log_context, &local_client, &table_name, &content).is_ok());
        }
    }

}
