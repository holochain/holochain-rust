use holochain_core_types::cas::content::Address;

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub tag: String,
}

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct GetLinksResult {
    pub addresses: Vec<Address>,
}
