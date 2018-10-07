use cas::content::Address;

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub tag: String,
}

impl GetLinksArgs {
    pub fn to_attribute_name(&self) -> String {
        format!("link:{}:{}", &self.entry_address, &self.tag)
    }
}
