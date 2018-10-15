use cas::content::Address;
use serde_json;

//-------------------------------------------------------------------------------------------------
// Link
//-------------------------------------------------------------------------------------------------

pub type LinkTag = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LinkAdd {
    base: Address,
    target: Address,
    tag: LinkTag,
}

impl LinkAdd {
    pub fn new(base: &Address, target: &Address, tag: &str) -> Self {
        LinkAdd {
            base: base.to_owned(),
            target: target.to_owned(),
            tag: tag.to_owned(),
        }
    }

    // Getters
    pub fn base(&self) -> &Address {
        &self.base
    }

    pub fn target(&self) -> &Address {
        &self.target
    }

    pub fn tag(&self) -> &LinkTag {
        &self.tag
    }
}

//-------------------------------------------------------------------------------------------------
// LinkEntry
//-------------------------------------------------------------------------------------------------

// HC.LinkAction sync with hdk-rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LinkActionKind {
    ADD,
    DELETE,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LinkEntry {
    action_kind: LinkActionKind,
    link: LinkAdd,
}

impl LinkEntry {
    pub fn new(action_kind: LinkActionKind, base: &Address, target: &Address, tag: &str) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: LinkAdd::new(base, target, tag),
        }
    }

    pub fn action_kind(&self) -> &LinkActionKind {
        &self.action_kind
    }

    pub fn link(&self) -> &LinkAdd {
        &self.link
    }

    pub fn from_link(action_kind: LinkActionKind, link: &LinkAdd) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: link.clone(),
        }
    }
}

impl ToString for LinkEntry {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("LinkEntry failed to serialize")
    }
}

impl From<String> for LinkEntry {
    fn from(s: String) -> LinkEntry {
        serde_json::from_str(&s).expect("LinkEntry failed to deserialize")
    }
}

#[cfg(test)]
pub mod tests {

    use cas::content::AddressableContent;
    use entry::{
        entry_type::EntryType,
        link::{Link, LinkActionKind, LinkEntry, LinkTag},
        test_entry_a, test_entry_b, Entry, ToEntry,
    };
    use std::string::ToString;

    pub fn test_link_tag() -> LinkTag {
        LinkTag::from("foo-tag")
    }

    pub fn test_link() -> Link {
        Link::new(
            &test_entry_a().address(),
            &test_entry_b().address(),
            &test_link_tag(),
        )
    }

    pub fn test_link_entry_action_kind() -> LinkActionKind {
        LinkActionKind::ADD
    }

    pub fn test_link_entry() -> LinkEntry {
        let link = test_link();
        LinkEntry::new(
            test_link_entry_action_kind(),
            link.base(),
            link.target(),
            link.tag(),
        )
    }

    pub fn test_link_entry_string() -> String {
        format!(
            "{{\"action_kind\":\"ADD\",\"link\":{{\"base\":\"{}\",\"target\":\"{}\",\"tag\":\"foo-tag\"}}}}",
            test_entry_a().address(),
            test_entry_b().address(),
        )
    }

    #[test]
    fn link_smoke_test() {
        test_link();
    }

    #[test]
    fn link_base_test() {
        assert_eq!(&test_entry_a().address(), test_link().base(),);
    }

    #[test]
    fn link_target_test() {
        assert_eq!(&test_entry_b().address(), test_link().target(),);
    }

    #[test]
    fn link_tag_test() {
        assert_eq!(&test_link_tag(), test_link().tag(),);
    }

    #[test]
    fn link_entry_smoke_test() {
        test_link_entry();
    }

    #[test]
    fn link_entry_action_kind_test() {
        assert_eq!(
            &test_link_entry_action_kind(),
            test_link_entry().action_kind(),
        );
    }

    #[test]
    fn link_entry_link_test() {
        assert_eq!(&test_link(), test_link_entry().link(),);
    }

    #[test]
    /// show ToString for LinkEntry
    fn link_entry_to_string_test() {
        assert_eq!(test_link_entry_string(), test_link_entry().to_string(),);
    }

    #[test]
    /// show From<String> for LinkEntry
    fn link_entry_from_string_test() {
        assert_eq!(LinkEntry::from(test_link_entry_string()), test_link_entry(),);
    }

    #[test]
    /// show ToEntry implementation for Link
    fn link_entry_to_entry_test() {
        // to_entry()
        assert_eq!(
            Entry::new(&EntryType::Link, &test_link_entry_string()),
            test_link_entry().to_entry(),
        );

        // from_entry()
        assert_eq!(
            test_link_entry(),
            LinkEntry::from_entry(&test_link_entry().to_entry()),
        );
    }
}
