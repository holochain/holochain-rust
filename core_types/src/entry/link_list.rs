use entry::link_add::LinkAdd;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct LinkList {
    links: Vec<LinkAdd>,
}

impl LinkList {
    pub fn new(links: &[LinkAdd]) -> Self {
        LinkList {
            links: links.to_vec(),
        }
    }
     pub fn links(&self) -> &Vec<LinkAdd> {
        &self.links
    }
}
