use crate::{
    dht::bbdht::{
        dynamodb::{api::aspect::read::get_entry_aspects, client::Client},
        error::{BbDhtError, BbDhtResult},
    },
    trace::{tracer, LogContext},
    workflow::state::Sim1hState,
};
use holochain_core_types::network::query::NetworkQuery;
use holochain_json_api::json::JsonString;
use lib3h_protocol::{
    data_types::{EntryAspectData, Opaque, QueryEntryData},
    protocol::Lib3hToClient,
};
use std::convert::TryFrom;

pub fn get_entry_aspect_filter_fn(aspect: &EntryAspectData) -> bool {
    let keep = vec!["content".to_string(), "header".to_string()];
    keep.contains(&aspect.type_hint)
}

pub fn query_entry_aspects(
    log_context: &LogContext,
    client: &Client,
    query_entry_data: &QueryEntryData,
) -> BbDhtResult<Vec<EntryAspectData>> {
    tracer(&log_context, "publish_entry");

    let table_name = query_entry_data.space_address.to_string();
    let entry_address = query_entry_data.entry_address.clone();

    let query_raw = query_entry_data.query.as_slice();
    let utf8_result = std::str::from_utf8(&(*query_raw));
    let query_str = match utf8_result {
        Ok(v) => v,
        Err(err) => return Err(BbDhtError::CorruptData(err.to_string())),
    };
    let query_json = JsonString::from_json(&query_str.to_string());
    let _query = match NetworkQuery::try_from(query_json) {
        Ok(v) => v,
        Err(err) => return Err(BbDhtError::CorruptData(err.to_string())),
    };

    let entry_aspects = get_entry_aspects(log_context, client, &table_name, &entry_address)?;

    Ok(entry_aspects)
}

pub fn aspects_to_opaque(aspects: &Vec<EntryAspectData>) -> Opaque {
    let json = JsonString::from(aspects.clone());
    json.to_bytes().into()
}

impl Sim1hState {
    /// 90% (need query logic to be finalised)
    /// fetch all entry aspects from entry address
    /// do some kind of filter based on the non-opaque query struct
    /// familiar to rehydrate the opaque query struct
    pub fn query_entry(
        &mut self,
        log_context: &LogContext,
        _client: &Client,
        query_entry_data: &QueryEntryData,
    ) -> BbDhtResult<()> {
        tracer(&log_context, "query_entry");

        // Just mirror the request back, since we are a full-sync bbDHT
        self.client_request_outbox
            .push(Lib3hToClient::HandleQueryEntry(query_entry_data.clone()));

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{
        aspect::{
            entry_aspect_to_entry_aspect_data,
            fixture::{
                content_aspect_fresh, deletion_aspect_fresh, header_aspect_fresh,
                link_add_aspect_fresh, link_remove_aspect_fresh, update_aspect_fresh,
            },
        },
        dht::bbdht::dynamodb::client::local::local_client,
        entry::fixture::{entry_fresh, entry_hash_fresh},
        space::fixture::space_data_fresh,
        test::unordered_vec_compare,
        trace::tracer,
        workflow::{
            from_client::{
                fixture::{provided_entry_data_fresh, query_entry_data_fresh},
                publish_entry::publish_entry,
                query_entry::{get_entry_aspect_filter_fn, query_entry_aspects},
            },
            state::Sim1hState,
        },
    };

    #[test]
    pub fn get_entry_aspect_filter_fn_test() {
        // things that should persist
        assert!(get_entry_aspect_filter_fn(
            &entry_aspect_to_entry_aspect_data(content_aspect_fresh())
        ));
        assert!(get_entry_aspect_filter_fn(
            &entry_aspect_to_entry_aspect_data(header_aspect_fresh(&entry_fresh()))
        ));

        // things that should be dropped
        assert!(!get_entry_aspect_filter_fn(
            &entry_aspect_to_entry_aspect_data(link_add_aspect_fresh(&entry_fresh()))
        ));
        assert!(!get_entry_aspect_filter_fn(
            &entry_aspect_to_entry_aspect_data(link_remove_aspect_fresh(&entry_fresh()))
        ));
        assert!(!get_entry_aspect_filter_fn(
            &entry_aspect_to_entry_aspect_data(update_aspect_fresh(&entry_fresh()))
        ));
        assert!(!get_entry_aspect_filter_fn(
            &entry_aspect_to_entry_aspect_data(deletion_aspect_fresh(&entry_fresh()))
        ));
    }

    #[test]
    pub fn query_entry_aspects_test() {
        let log_context = "query_entry_aspects_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let space_data = space_data_fresh();
        let entry_address = entry_hash_fresh();
        let query_entry_data = query_entry_data_fresh(&space_data, &entry_address);
        let provided_entry_data = provided_entry_data_fresh(&space_data, &entry_address);

        // join space
        assert!(Sim1hState::join_space(&log_context, &local_client, &space_data).is_ok());

        // publish entry
        assert!(publish_entry(&log_context, &local_client, &provided_entry_data).is_ok());

        match query_entry_aspects(&log_context, &local_client, &query_entry_data) {
            Ok(v) => assert!(unordered_vec_compare(
                v,
                provided_entry_data.entry.aspect_list
            )),
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }
}
