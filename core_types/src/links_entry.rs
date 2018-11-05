use cas::content::Address;
use entry::{Entry, ToEntry};
use entry_type::EntryType;
use error::HolochainError;
use json::JsonString;
use std::convert::TryInto;

//-------------------------------------------------------------------------------------------------
// Link
//-------------------------------------------------------------------------------------------------

type LinkTag = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, DefaultJson)]
pub struct Link {
    base: Address,
    target: Address,
    tag: LinkTag,
}

impl Link {
    pub fn new(base: &Address, target: &Address, tag: &str) -> Self {
        Link {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct LinkEntry {
    action_kind: LinkActionKind,
    link: Link,
}

impl LinkEntry {
    pub fn new(action_kind: LinkActionKind, base: &Address, target: &Address, tag: &str) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: Link::new(base, target, tag),
        }
    }

    pub fn action_kind(&self) -> &LinkActionKind {
        &self.action_kind
    }

    pub fn link(&self) -> &Link {
        &self.link
    }

    pub fn from_link(action_kind: LinkActionKind, link: &Link) -> Self {
        LinkEntry {
            action_kind: action_kind,
            link: link.clone(),
        }
    }
}

impl ToEntry for LinkEntry {
    // Convert a LinkEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        Entry::new(EntryType::Link, self.to_owned())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::Link, entry.entry_type());
        entry
            .value()
            .to_owned()
            .try_into()
            .expect("could not convert Entry to LinkEntry")
    }
}

//-------------------------------------------------------------------------------------------------
// LinkListEntry
//-------------------------------------------------------------------------------------------------
//
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, DefaultJson)]
pub struct LinkListEntry {
    links: Vec<Link>,
}

impl LinkListEntry {
    pub fn new(links: &[Link]) -> Self {
        LinkListEntry {
            links: links.to_vec(),
        }
    }

    pub fn links(&self) -> &Vec<Link> {
        &self.links
    }
}

impl ToEntry for LinkListEntry {
    // Convert a LinkListEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        Entry::new(EntryType::LinkList, self.to_owned())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::LinkList, entry.entry_type());
        entry
            .value()
            .to_owned()
            .try_into()
            .expect("could not convert Entry to LinkListEntry")
    }
}

#[cfg(test)]
pub mod tests {

    use cas::content::AddressableContent;
    use entry::{test_entry_a, test_entry_b, Entry, ToEntry};
    use entry_type::EntryType;
    use json::JsonString;
    use links_entry::{Link, LinkActionKind, LinkEntry, LinkTag};

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

    pub fn test_link_entry_json_string() -> JsonString {
        JsonString::from(format!(
            "{{\"action_kind\":\"ADD\",\"link\":{{\"base\":\"{}\",\"target\":\"{}\",\"tag\":\"foo-tag\"}}}}",
            test_entry_a().address(),
            test_entry_b().address(),
        ))
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
        assert_eq!(
            test_link_entry_json_string(),
            JsonString::from(test_link_entry()),
        );
    }

    #[test]
    /// show From<String> for LinkEntry
    fn link_entry_from_string_test() {
        assert_eq!(
            LinkEntry::from(test_link_entry_json_string()),
            test_link_entry(),
        );
    }

    #[test]
    /// show ToEntry implementation for Link
    fn link_entry_to_entry_test() {
        // to_entry()
        assert_eq!(
            Entry::new(EntryType::Link, test_link_entry_json_string()),
            test_link_entry().to_entry(),
        );

        // from_entry()
        assert_eq!(
            test_link_entry(),
            LinkEntry::from_entry(&test_link_entry().to_entry()),
        );
    }
}
