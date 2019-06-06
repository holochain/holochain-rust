use lib3h_persistence_api::{cas::content::Address, error::PersistenceError, json::*};

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct ReceiveParams {
    pub from: Address,
    pub payload: String,
}
