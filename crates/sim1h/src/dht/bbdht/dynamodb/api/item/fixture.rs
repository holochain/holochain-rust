use holochain_json_api::json::RawString;
use holochain_persistence_api::cas::content::Content;
use uuid::Uuid;

pub fn content_fresh() -> Content {
    Content::from(RawString::from(Uuid::new_v4().to_string()))
}
