use crate::{
    dht::bbdht::{
        dynamodb::{api::table::exist::until_table_not_exists, client::Client},
        error::BbDhtResult,
    },
    trace::{tracer, LogContext},
};
use rusoto_dynamodb::{DeleteTableInput, DynamoDb};

pub fn delete_table(
    log_context: &LogContext,
    client: &Client,
    table_name: &str,
) -> BbDhtResult<()> {
    tracer(&log_context, "delete_table");
    let delete_table_input = DeleteTableInput {
        table_name: table_name.to_string(),
    };
    client.delete_table(delete_table_input).sync()?;
    until_table_not_exists(log_context, client, table_name);
    Ok(())
}

#[cfg(test)]
pub mod test {

    use crate::{
        dht::bbdht::dynamodb::{
            api::table::{
                create::create_table, delete::delete_table, exist::table_exists,
                fixture::table_name_fresh,
            },
            client::local::local_client,
            schema::fixture::{attribute_definitions_a, key_schema_a},
        },
        trace::tracer,
    };

    #[test]
    fn delete_table_test() {
        let log_context = "delete_table_text";

        tracer(&log_context, "fixtures");

        let local_client = local_client();
        let table_name = table_name_fresh();
        let key_schema = key_schema_a();
        let attribute_definitions = attribute_definitions_a();

        // not exists
        assert!(!table_exists(&log_context, &local_client, &table_name)
            .expect("could not check that table exists"));

        // create
        assert!(create_table(
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

        // delete
        assert!(delete_table(&log_context, &local_client, &table_name).is_ok());

        // not exists
        assert!(!table_exists(&log_context, &local_client, &table_name)
            .expect("could not check that the table exists"));
    }

}
