use crate::{
    dht::bbdht::{
        dynamodb::{
            api::table::exist::{table_exists, until_table_exists},
            client::Client,
            schema::cas::{attribute_definitions_cas, key_schema_cas},
        },
        error::{BbDhtError, BbDhtResult},
    },
    trace::{tracer, LogContext},
};
use dynomite::dynamodb::CreateTableInput;
use rusoto_dynamodb::{
    AttributeDefinition, DynamoDb, KeySchemaElement, ProvisionedThroughput, TableDescription,
};

pub fn create_table(
    log_context: &LogContext,
    client: &Client,
    table_name: &str,
    key_schema: &Vec<KeySchemaElement>,
    attribute_definitions: &Vec<AttributeDefinition>,
) -> BbDhtResult<Option<TableDescription>> {
    tracer(&log_context, &format!("create_table {}", table_name));

    let create_table_input = CreateTableInput {
        table_name: table_name.to_string(),
        key_schema: key_schema.clone(),
        attribute_definitions: attribute_definitions.clone(),
        provisioned_throughput: Some(ProvisionedThroughput {
            read_capacity_units: 1,
            write_capacity_units: 1,
        }),
        ..Default::default()
    };

    let output = client.create_table(create_table_input).sync()?;
    until_table_exists(log_context, client, table_name);
    Ok(output.table_description)
}

pub fn ensure_table(
    log_context: &LogContext,
    client: &Client,
    table_name: &str,
    key_schema: &Vec<KeySchemaElement>,
    attribute_definitions: &Vec<AttributeDefinition>,
) -> BbDhtResult<Option<TableDescription>> {
    tracer(log_context, &format!("ensure_table {}", &table_name));

    // well in reality we end up with concurrency issues if we do a list or describe
    // there is a specific error returned for a table that already exists so we defuse to None
    match table_exists(log_context, client, table_name) {
        Ok(false) => match create_table(
            log_context,
            client,
            table_name,
            key_schema,
            attribute_definitions,
        ) {
            Ok(created) => Ok(created),
            Err(BbDhtError::ResourceInUse(_)) => {
                tracer(&log_context, "ensure_table ResourceInUse");
                Ok(None)
            }
            Err(_err) => {
                tracer(&log_context, "ensure_table failed to create table. retry.");
                ensure_table(
                    &log_context,
                    &client,
                    &table_name,
                    &key_schema,
                    &attribute_definitions,
                )
            }
        },
        Ok(true) => Ok(None),
        Err(BbDhtError::InternalServerError(_)) => {
            tracer(&log_context, "retry ensure_table InternalServerError");
            // RusotoError::Service(CreateTableError::InternalServerError(err)),
            ensure_table(
                &log_context,
                &client,
                &table_name,
                &key_schema,
                &attribute_definitions,
            )
        }
        Err(BbDhtError::Unknown(_)) => {
            tracer(&log_context, "retry ensure_table Unknown");
            ensure_table(
                &log_context,
                &client,
                &table_name,
                &key_schema,
                &attribute_definitions,
            )
        }
        Err(BbDhtError::ResourceNotFound(_)) => {
            // this must be covered by table_exists
            tracer(&log_context, "retry ensure_table ResourceNotFound");
            ensure_table(
                &log_context,
                &client,
                &table_name,
                &key_schema,
                &attribute_definitions,
            )
        }
        Err(err) => Err(err),
    }
}

pub fn ensure_cas_table(
    log_context: &LogContext,
    client: &Client,
    table_name: &str,
) -> BbDhtResult<Option<TableDescription>> {
    tracer(&log_context, &format!("ensure_cas_table {}", &table_name));

    ensure_table(
        log_context,
        client,
        table_name,
        &key_schema_cas(),
        &attribute_definitions_cas(),
    )
}

#[cfg(test)]
pub mod test {
    use crate::dht::bbdht::dynamodb::api::table::create::{create_table, ensure_table};

    use crate::{
        dht::bbdht::dynamodb::{
            api::table::{
                create::ensure_cas_table, describe::describe_table, exist::table_exists,
                fixture::table_name_fresh,
            },
            client::local::local_client,
            schema::{
                cas::{attribute_definitions_cas, key_schema_cas},
                fixture::{attribute_definitions_a, key_schema_a},
            },
        },
        trace::tracer,
    };

    #[test]
    fn create_table_test() {
        let log_context = "create_table_test";

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
    }

    #[test]
    fn ensure_table_test() {
        let log_context = "ensure_table_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let key_schema = key_schema_a();
        let attribute_definitions = attribute_definitions_a();

        // not exists
        assert!(!table_exists(&log_context, &local_client, &table_name).unwrap());

        // ensure
        assert!(ensure_table(
            &log_context,
            &local_client,
            &table_name,
            &key_schema,
            &attribute_definitions,
        )
        .is_ok());

        // exists
        assert!(table_exists(&log_context, &local_client, &table_name).unwrap());

        // ensure again
        assert_eq!(
            None,
            ensure_table(
                &log_context,
                &local_client,
                &table_name,
                &key_schema,
                &attribute_definitions,
            )
            .expect("could not check table")
        );

        // exists
        assert!(table_exists(&log_context, &local_client, &table_name).unwrap());
    }

    #[test]
    fn ensure_cas_table_test() {
        let log_context = "ensure_cas_table_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // check cas schema
        match describe_table(&log_context, &local_client, &table_name) {
            Ok(table_description) => {
                assert_eq!(Some(key_schema_cas()), table_description.key_schema);
                assert_eq!(
                    Some(attribute_definitions_cas()),
                    table_description.attribute_definitions
                );
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }

        // thrash
        for _ in 0..100 {
            assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());
        }
    }

}
