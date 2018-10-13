use cas::content::{Address, AddressableContent};
use entry::{Entry, ToEntry};
use entry_type::EntryType;
use serde_json;

//-------------------------------------------------------------------------------------------------
// Link
//-------------------------------------------------------------------------------------------------

type LinkTag = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LinkActionKind {
    ADD,
    DELETE,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
        let json_array = serde_json::to_string(self).expect("LinkEntry should serialize");
        Entry::new(&EntryType::Link, &json_array)
    }

    fn from_entry(entry: &Entry) -> Self {
        serde_json::from_str(&entry.content()).expect("entry is not a valid LinkEntry")
    }
}

//-------------------------------------------------------------------------------------------------
// LinkListEntry
//-------------------------------------------------------------------------------------------------
//
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct LinkListEntry {
    pub links: Vec<Link>,
}

impl LinkListEntry {
    pub fn new(links: &[Link]) -> Self {
        LinkListEntry {
            links: links.to_vec(),
        }
    }
}

impl ToEntry for LinkListEntry {
    // Convert a LinkListEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        let json_array = serde_json::to_string(self).expect("LinkListEntry failed to serialize");
        Entry::new(&EntryType::LinkList, &Entry::from(json_array))
    }

    fn from_entry(entry: &Entry) -> Self {
        serde_json::from_str(&entry.content()).expect("entry failed converting into LinkListEntry")
    }
}

#[cfg(test)]
pub mod tests {

    use links_entry::LinkTag;
    use links_entry::Link;
    use entry::test_entry_a;
    use entry::test_entry_b;
    use cas::content::AddressableContent;

    pub fn test_link_tag() -> LinkTag {
        LinkTag::from("foo-tag")
    }

    pub fn test_link() -> Link {
        Link::new(&test_entry_a().address(), &test_entry_b().address(), &test_link_tag())
    }

    #[test]
    fn link_smoke_test() {
        test_link();
    }

    #[test]
    fn link_base_test() {
        assert_eq!(
            &test_entry_a().address(),
            test_link().base(),
        );
    }

    #[test]
    fn link_target_test() {
        assert_eq!(
            &test_entry_b().address(),
            test_link().target(),
        );
    }

    #[test]
    fn link_tag_test() {
        assert_eq!(
            &test_link_tag(),
            test_link().tag(),
        );
    }
}
