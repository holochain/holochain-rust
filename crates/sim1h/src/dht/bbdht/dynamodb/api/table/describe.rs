use crate::dht::bbdht::dynamodb::client::Client;
use crate::dht::bbdht::error::BbDhtError;
use crate::dht::bbdht::error::BbDhtResult;
use crate::trace::tracer;
use crate::trace::LogContext;
use rusoto_dynamodb::DescribeTableInput;
use rusoto_dynamodb::DynamoDb;
use rusoto_dynamodb::TableDescription;

pub fn describe_table(
    log_context: &LogContext,
    client: &Client,
    table_name: &str,
) -> BbDhtResult<TableDescription> {
    tracer(&log_context, &format!("describe_table {}", &table_name));
    match client
        .describe_table(DescribeTableInput {
            table_name: table_name.to_string(),
        })
        .sync()?
        .table
    {
        Some(table_description) => Ok(table_description),
        None => Err(BbDhtError::ResourceNotFound(String::from(
            "None returned for table description",
        ))),
    }
}

#[cfg(test)]
pub mod test {

    use crate::dht::bbdht::dynamodb::api::table::create::ensure_table;
    use crate::dht::bbdht::dynamodb::api::table::describe::describe_table;
    use crate::dht::bbdht::dynamodb::api::table::exist::table_exists;
    use crate::dht::bbdht::dynamodb::api::table::fixture::table_name_fresh;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::dht::bbdht::dynamodb::schema::fixture::attribute_definitions_a;
    use crate::dht::bbdht::dynamodb::schema::fixture::key_schema_a;
    use crate::dht::bbdht::error::BbDhtError;
    use crate::trace::tracer;

    #[test]
    fn describe_table_test() {
        let log_context = "describe_table_test";

        tracer(&log_context, "describe_table_test");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let key_schema = key_schema_a();
        let attribute_definitions = attribute_definitions_a();

        // not exists
        assert!(!table_exists(&log_context, &local_client, &table_name)
            .expect("could not check that table exists"));

        // ensure table
        assert!(ensure_table(
            &log_context,
            &local_client,
            &table_name,
            &key_schema,
            &attribute_definitions,
        )
        .is_ok());

        // exists
        assert!(table_exists(&log_context, &local_client, &table_name)
            .expect("could not check that table exists"));

        // active
        assert_eq!(
            Some(String::from("ACTIVE")),
            describe_table(&log_context, &local_client, &table_name)
                .expect("could not describe table")
                .table_status,
        );
    }

    #[test]
    fn describe_table_missing_test() {
        let log_context = "describe_table_missing_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();

        // missing description error
        assert_eq!(
            Err(BbDhtError::ResourceNotFound(String::from(
                "Cannot do operations on a non-existent table"
            ))),
            describe_table(&log_context, &local_client, &table_name),
        );
    }

}
