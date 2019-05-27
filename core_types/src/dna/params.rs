use crate::json::JsonString;

pub type GenesisParams = JsonString;
pub type InitParams = JsonString;

#[derive(Clone, Default, Debug)]
pub struct DnaParams {
    pub genesis: GenesisParams,
    pub init: InitParams,
}
