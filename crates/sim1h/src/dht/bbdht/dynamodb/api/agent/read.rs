use crate::{
    dht::bbdht::{
        dynamodb::{
            api::{item::read::get_item_by_address, table::exist::table_exists},
            client::Client,
            schema::TableName,
        },
        error::BbDhtResult,
    },
    trace::{tracer, LogContext},
};
use lib3h_protocol::types::AgentPubKey;

pub fn agent_exists(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    agent_id: &AgentPubKey,
) -> BbDhtResult<bool> {
    tracer(&log_context, "agent_exists");

    // agent only exists in the space if the space exists
    Ok(if table_exists(log_context, client, table_name)? {
        get_item_by_address(log_context, client, table_name, agent_id)?.is_some()
    } else {
        false
    })
}

#[cfg(test)]
pub mod tests {

    use crate::{
        agent::fixture::agent_id_fresh,
        dht::bbdht::dynamodb::{
            api::{
                agent::{read::agent_exists, write::touch_agent},
                table::{create::ensure_cas_table, fixture::table_name_fresh},
            },
            client::local::local_client,
        },
        trace::tracer,
    };

    #[test]
    fn agent_exists_test() {
        let log_context = "agent_exists";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let agent_id = agent_id_fresh();

        // agent not exists if space not exists
        match agent_exists(&log_context, &local_client, &table_name, &agent_id) {
            Ok(false) => {
                tracer(&log_context, "👌");
            }
            Ok(true) => {
                panic!("apparently agent exists before the space does");
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        };

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // agent not exists if not join space
        match agent_exists(&log_context, &local_client, &table_name, &agent_id) {
            Ok(false) => {
                tracer(&log_context, "👌");
            }
            Ok(true) => {
                panic!("agent exists before join");
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        };

        // join
        assert!(touch_agent(&log_context, &local_client, &table_name, &agent_id).is_ok());

        // agent exists now
        match agent_exists(&log_context, &local_client, &table_name, &agent_id) {
            Ok(false) => {
                panic!("agent not exists after join");
            }
            Ok(true) => {
                tracer(&log_context, "👌");
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }
}
