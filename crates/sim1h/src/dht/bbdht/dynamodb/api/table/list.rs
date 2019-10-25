use crate::dht::bbdht::dynamodb::client::Client;
use crate::dht::bbdht::dynamodb::schema::TableName;
use crate::dht::bbdht::error::BbDhtResult;
use dynomite::dynamodb::{DynamoDb, ListTablesInput};

pub fn list_tables(client: &Client) -> BbDhtResult<Option<Vec<TableName>>> {
    Ok(client
        .list_tables(ListTablesInput {
            ..Default::default()
        })
        .sync()?
        .table_names)
}

#[cfg(test)]
pub mod test {
    use crate::dht::bbdht::dynamodb::api::table::list::list_tables;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::trace::tracer;

    #[test]
    pub fn list_tables_test() {
        let log_context = "list_tables_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();

        // list
        assert!(list_tables(&local_client).is_ok());
    }
}
