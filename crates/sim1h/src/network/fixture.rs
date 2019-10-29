use holochain_core_types::time::{test_iso_8601, Iso8601};
use lib3h_protocol::data_types::Opaque;
use uuid::Uuid;

pub fn request_id_fresh() -> String {
    Uuid::new_v4().to_string()
}

pub fn opaque_fresh() -> Opaque {
    vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9].into()
}

pub fn timestamp_fresh() -> Iso8601 {
    test_iso_8601()
}
