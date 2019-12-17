use crate::{
    dht::bbdht::{
        dynamodb::{
            api::item::write::should_put_item_retry,
            client::Client,
            schema::{cas::ADDRESS_KEY, string_attribute_value, TableName},
        },
        error::BbDhtResult,
    },
    trace::{tracer, LogContext},
};
use lib3h_protocol::types::AgentPubKey;
use rusoto_dynamodb::{DynamoDb, PutItemInput};
use std::collections::HashMap;

pub fn touch_agent(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    agent_id: &AgentPubKey,
) -> BbDhtResult<()> {
    tracer(&log_context, "touch_agent");

    let mut item = HashMap::new();
    item.insert(
        String::from(ADDRESS_KEY),
        string_attribute_value(&String::from(agent_id.to_owned())),
    );

    if should_put_item_retry(
        log_context,
        client
            .put_item(PutItemInput {
                table_name: table_name.to_string(),
                item,
                ..Default::default()
            })
            .sync(),
    )? {
        touch_agent(log_context, client, table_name, agent_id)
    } else {
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{
        agent::fixture::agent_id_fresh,
        dht::bbdht::dynamodb::{
            api::{
                agent::write::touch_agent,
                table::{create::ensure_cas_table, exist::table_exists, fixture::table_name_fresh},
            },
            client::local::local_client,
        },
        trace::tracer,
    };

    #[test]
    fn touch_agent_test() {
        let log_context = "touch_agent_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let agent_id = agent_id_fresh();

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // cas exists
        assert!(table_exists(&log_context, &local_client, &table_name).is_ok());

        // touch agent
        assert!(touch_agent(&log_context, &local_client, &table_name, &agent_id).is_ok());
    }
}
