use crate::{
    dht::bbdht::{
        dynamodb::{
            api::item::{write::should_put_item_retry, Item},
            client::Client,
            schema::{
                blob_attribute_value, bool_attribute_value,
                cas::{
                    inbox_key, ADDRESS_KEY, MESSAGE_CONTENT_KEY, MESSAGE_FROM_KEY,
                    MESSAGE_IS_RESPONSE_KEY, MESSAGE_SPACE_ADDRESS_KEY, MESSAGE_TO_KEY,
                    REQUEST_IDS_KEY, REQUEST_IDS_SEEN_KEY,
                },
                string_attribute_value, string_set_attribute_value, TableName,
            },
        },
        error::{BbDhtError, BbDhtResult},
    },
    trace::{tracer, LogContext},
};
use holochain_persistence_api::{cas::content::Address, hash::HashString};
use lib3h_protocol::{data_types::DirectMessageData, types::AgentPubKey};
use rusoto_dynamodb::{DynamoDb, GetItemInput, PutItemInput, UpdateItemInput};
use std::collections::HashMap;

/// put an item that can be reconstructed to DirectMessageData against the request id
#[allow(clippy::too_many_arguments)]
pub fn put_inbox_message(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    request_id: &String,
    from: &Address,
    to: &Address,
    content: &Vec<u8>,
    response: bool,
) -> BbDhtResult<()> {
    tracer(&log_context, "put_inbox_message");

    let mut message_item = HashMap::new();

    message_item.insert(
        String::from(ADDRESS_KEY),
        string_attribute_value(request_id),
    );

    message_item.insert(
        String::from(MESSAGE_FROM_KEY),
        string_attribute_value(&from.to_string()),
    );

    message_item.insert(
        String::from(MESSAGE_TO_KEY),
        string_attribute_value(&to.to_string()),
    );

    message_item.insert(
        String::from(MESSAGE_SPACE_ADDRESS_KEY),
        string_attribute_value(&table_name.to_string()),
    );

    message_item.insert(
        String::from(MESSAGE_CONTENT_KEY),
        blob_attribute_value(&content),
    );

    message_item.insert(
        String::from(MESSAGE_IS_RESPONSE_KEY),
        bool_attribute_value(response),
    );

    if should_put_item_retry(
        log_context,
        client
            .put_item(PutItemInput {
                table_name: table_name.to_string(),
                item: message_item,
                ..Default::default()
            })
            .sync(),
    )? {
        put_inbox_message(
            log_context,
            client,
            table_name,
            request_id,
            from,
            to,
            content,
            response,
        )
    } else {
        Ok(())
    }
}

pub fn append_request_id_to_inbox(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    folder: &String,
    request_id: &String,
    to: &Address,
) -> BbDhtResult<()> {
    tracer(&log_context, "append_request_id_to_inbox");

    let mut inbox_address_key = HashMap::new();

    // primary key is the inbox name "inbox_<agent_id>"
    inbox_address_key.insert(
        String::from(ADDRESS_KEY),
        string_attribute_value(&inbox_key(to)),
    );

    // the request id appended under the inbox address
    let mut inbox_attribute_values = HashMap::new();
    inbox_attribute_values.insert(
        ":request_ids".to_string(),
        string_set_attribute_value(vec![request_id.to_string()]),
    );

    let mut inbox_attribute_names = HashMap::new();
    inbox_attribute_names.insert("#request_ids".to_string(), folder.to_string());

    // https://stackoverflow.com/questions/31288085/how-to-append-a-value-to-list-attribute-on-aws-dynamodb
    let update_expression = "ADD #request_ids :request_ids";

    let request_ids_update = UpdateItemInput {
        table_name: table_name.to_string(),
        key: inbox_address_key,
        update_expression: Some(update_expression.to_string()),
        expression_attribute_names: Some(inbox_attribute_names),
        expression_attribute_values: Some(inbox_attribute_values),
        ..Default::default()
    };

    client.update_item(request_ids_update).sync()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn send_to_agent_inbox(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    request_id: &String,
    from: &Address,
    to: &Address,
    content: &Vec<u8>,
    response: bool,
) -> BbDhtResult<()> {
    tracer(&log_context, "send_to_agent_inbox");

    put_inbox_message(
        log_context,
        client,
        table_name,
        request_id,
        from,
        to,
        content,
        response,
    )?;

    append_request_id_to_inbox(
        log_context,
        client,
        table_name,
        &REQUEST_IDS_KEY.to_string(),
        request_id,
        to,
    )?;

    Ok(())
}

pub fn get_inbox_request_ids(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    inbox_folder: &String,
    to: &Address,
) -> BbDhtResult<Vec<String>> {
    tracer(log_context, "get_inbox_request_ids");

    let mut key = HashMap::new();
    key.insert(
        String::from(ADDRESS_KEY),
        string_attribute_value(&inbox_key(to)),
    );
    let get_item_output = client
        .get_item(GetItemInput {
            consistent_read: Some(true),
            table_name: table_name.into(),
            key: key,
            ..Default::default()
        })
        .sync()?
        .item;
    Ok(match get_item_output {
        Some(item) => match item.get(inbox_folder) {
            Some(attribute) => match attribute.ss.clone() {
                Some(ss) => ss,
                None => Vec::new(),
            },
            None => Vec::new(),
        },
        None => Vec::new(),
    })
}

pub fn item_to_direct_message_data(item: &Item) -> BbDhtResult<(DirectMessageData, bool)> {
    let content = match item[MESSAGE_CONTENT_KEY].b.clone() {
        Some(v) => v.to_vec(),
        None => {
            return Err(BbDhtError::MissingData(format!(
                "message item missing content {:?}",
                &item
            )))
        }
    };

    let from_agent_id = match item[MESSAGE_FROM_KEY].s.clone() {
        Some(v) => v,
        None => {
            return Err(BbDhtError::MissingData(format!(
                "message item missing from {:?}",
                &item
            )))
        }
    };

    let to_agent_id = match item[MESSAGE_TO_KEY].s.clone() {
        Some(v) => v,
        None => {
            return Err(BbDhtError::MissingData(format!(
                "message item missing to {:?}",
                &item
            )))
        }
    };

    let space_address = match item[MESSAGE_SPACE_ADDRESS_KEY].s.clone() {
        Some(v) => v,
        None => {
            return Err(BbDhtError::MissingData(format!(
                "message item missing space_address {:?}",
                &item
            )))
        }
    };

    let request_id = match item[ADDRESS_KEY].s.clone() {
        Some(v) => v,
        None => {
            return Err(BbDhtError::MissingData(format!(
                "message item missing request_id {:?}",
                &item
            )))
        }
    };

    let is_response = match item[MESSAGE_IS_RESPONSE_KEY].bool {
        Some(v) => v,
        None => {
            return Err(BbDhtError::MissingData(format!(
                "message item missing response flag {:?}",
                &item
            )))
        }
    };

    Ok((
        DirectMessageData {
            content: content.into(),
            from_agent_id: from_agent_id.into(),
            to_agent_id: to_agent_id.into(),
            request_id: request_id,
            space_address: HashString::from(space_address).into(),
        },
        is_response,
    ))
}

pub fn request_ids_to_messages(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    request_ids: &Vec<String>,
) -> BbDhtResult<Vec<(DirectMessageData, bool)>> {
    tracer(log_context, "request_ids_to_messages");

    let mut direct_message_datas = Vec::new();

    for request_id in request_ids {
        let mut key = HashMap::new();
        key.insert(
            String::from(ADDRESS_KEY),
            string_attribute_value(&request_id),
        );

        let get_item_output = client
            .get_item(GetItemInput {
                consistent_read: Some(true),
                table_name: table_name.into(),
                key: key,
                ..Default::default()
            })
            .sync()?
            .item;

        match get_item_output {
            Some(item) => {
                direct_message_datas.push(item_to_direct_message_data(&item)?);
            }
            // the request ids MUST be in the db
            None => {
                return Err(BbDhtError::MissingData(format!(
                    "missing message for request id: {:?}",
                    &request_id
                )))
            }
        }
    }

    Ok(direct_message_datas)
}

pub fn check_inbox(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    to: &AgentPubKey,
) -> BbDhtResult<Vec<(DirectMessageData, bool)>> {
    tracer(&log_context, "check_inbox");

    let inbox_request_ids = get_inbox_request_ids(
        log_context,
        client,
        table_name,
        &REQUEST_IDS_KEY.to_string(),
        to,
    )?;
    let seen_request_ids = get_inbox_request_ids(
        log_context,
        client,
        table_name,
        &REQUEST_IDS_SEEN_KEY.to_string(),
        to,
    )?;

    let unseen_request_ids: Vec<String> = inbox_request_ids
        .iter()
        .filter(|request_id| !seen_request_ids.contains(request_id))
        .cloned()
        .collect();

    let messages = request_ids_to_messages(log_context, client, table_name, &unseen_request_ids);

    // record that we have now seen the unseen without errors (so far)
    for unseen in unseen_request_ids.clone() {
        append_request_id_to_inbox(
            log_context,
            client,
            table_name,
            &REQUEST_IDS_SEEN_KEY.to_string(),
            &unseen,
            &to,
        )?;
    }

    messages
}

#[cfg(test)]
pub mod tests {

    use crate::{
        agent::fixture::{agent_id_fresh, message_content_fresh},
        dht::bbdht::dynamodb::{
            api::{
                agent::inbox::{
                    append_request_id_to_inbox, check_inbox, get_inbox_request_ids,
                    put_inbox_message, send_to_agent_inbox,
                },
                table::{create::ensure_cas_table, fixture::table_name_fresh},
            },
            client::local::local_client,
            schema::cas::{REQUEST_IDS_KEY, REQUEST_IDS_SEEN_KEY},
        },
        network::fixture::request_id_fresh,
        trace::tracer,
    };
    use holochain_persistence_api::hash::HashString;
    use lib3h_protocol::data_types::DirectMessageData;

    fn folders() -> Vec<String> {
        vec![
            REQUEST_IDS_KEY.to_string(),
            REQUEST_IDS_SEEN_KEY.to_string(),
        ]
    }

    #[test]
    fn append_request_id_to_inbox_test() {
        let log_context = "append_request_id_to_inbox_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let request_id = request_id_fresh();
        let to = agent_id_fresh();

        for folder in folders() {
            // ensure cas
            assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

            // append request_id
            assert!(append_request_id_to_inbox(
                &log_context,
                &local_client,
                &table_name,
                &folder,
                &request_id,
                &to
            )
            .is_ok());
        }
    }

    #[test]
    fn put_inbox_message_test() {
        let log_context = "put_inbox_message_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let request_id = request_id_fresh();
        let from = agent_id_fresh();
        let to = agent_id_fresh();
        let content = message_content_fresh();
        let is_response = false;

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // pub inbox message
        assert!(put_inbox_message(
            &log_context,
            &local_client,
            &table_name,
            &request_id,
            &from,
            &to,
            &content,
            is_response,
        )
        .is_ok());
    }

    #[test]
    fn send_to_agent_inbox_test() {
        let log_context = "send_to_agent_inbox_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let request_id = request_id_fresh();
        let from = agent_id_fresh();
        let to = agent_id_fresh();
        let content = message_content_fresh();
        let is_response = false;

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // pub inbox message
        assert!(send_to_agent_inbox(
            &log_context,
            &local_client,
            &table_name,
            &request_id,
            &from,
            &to,
            &content,
            is_response,
        )
        .is_ok());
    }

    #[test]
    fn get_inbox_request_ids_test() {
        let log_context = "get_inbox_request_ids_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let request_id = request_id_fresh();
        let from = agent_id_fresh();
        let to = agent_id_fresh();
        let content = message_content_fresh();
        let is_response = false;

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // pub inbox message
        assert!(send_to_agent_inbox(
            &log_context,
            &local_client,
            &table_name,
            &request_id.clone(),
            &from,
            &to,
            &content,
            is_response,
        )
        .is_ok());

        // get inbox message
        match get_inbox_request_ids(
            &log_context,
            &local_client,
            &table_name,
            &REQUEST_IDS_KEY.to_string(),
            &to,
        ) {
            Ok(request_ids) => assert_eq!(vec![request_id.clone()], request_ids),
            Err(err) => panic!("incorrect request id {:?}", err),
        };
    }

    #[test]
    fn check_inbox_test() {
        let log_context = "get_inbox_request_ids_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let request_id = request_id_fresh();
        let from = agent_id_fresh();
        let to = agent_id_fresh();
        let content = message_content_fresh();
        let is_response = false;

        let direct_message_data = DirectMessageData {
            content: content.clone().into(),
            from_agent_id: from.clone(),
            to_agent_id: to.clone(),
            request_id: request_id.clone(),
            space_address: HashString::from(table_name.clone()).into(),
        };

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // pub inbox message
        assert!(send_to_agent_inbox(
            &log_context,
            &local_client,
            &table_name,
            &request_id.clone(),
            &from,
            &to,
            &content,
            is_response,
        )
        .is_ok());

        // check inbox
        match check_inbox(&log_context, &local_client, &table_name, &to) {
            Ok(messages) => assert_eq!(vec![(direct_message_data.clone(), is_response)], messages),
            Err(err) => panic!("incorrect request id {:?}", err),
        };

        // check again, should be empty
        match check_inbox(&log_context, &local_client, &table_name, &to) {
            Ok(request_ids) => {
                let v: Vec<(DirectMessageData, bool)> = Vec::new();
                assert_eq!(v, request_ids);
            }
            Err(err) => panic!("incorrect request id {:?}", err),
        };
    }
}
