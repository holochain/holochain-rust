use crate::{
    cas::content::Address,
    entry::{entry_type::EntryType, Entry, ToEntry},
    error::HolochainError,
    json::JsonString,
    link::{Link, LinkActionKind},
};
use std::convert::TryInto;

//-------------------------------------------------------------------------------------------------
// LinkAddEntry
//-------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct LinkAddEntry {
    action_kind: LinkActionKind,
    link: Link,
}

impl LinkAddEntry {
    pub fn new(action_kind: LinkActionKind, base: &Address, target: &Address, tag: &str) -> Self {
        LinkAddEntry {
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
        LinkAddEntry {
            action_kind: action_kind,
            link: link.clone(),
        }
    }
}

impl ToEntry for LinkAddEntry {
    // Convert a LinkEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        Entry::new(EntryType::LinkAdd, self.to_owned())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::LinkAdd, entry.entry_type());
        entry
            .value()
            .to_owned()
            .try_into()
            .expect("could not convert Entry to LinkEntry")
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{
        cas::content::AddressableContent,
        entry::{entry_type::EntryType, test_entry_a, test_entry_b, Entry, ToEntry},
        json::JsonString,
        link::{
            link_add::LinkAddEntry,
            tests::{example_link, example_link_action_kind, example_link_tag},
        },
    };
    use std::convert::TryFrom;

    pub fn test_link_entry() -> LinkAddEntry {
        let link = example_link();
        LinkAddEntry::new(
            example_link_action_kind(),
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
        example_link();
    }

    #[test]
    fn link_base_test() {
        assert_eq!(&test_entry_a().address(), example_link().base(),);
    }

    #[test]
    fn link_target_test() {
        assert_eq!(&test_entry_b().address(), example_link().target(),);
    }

    #[test]
    fn link_tag_test() {
        assert_eq!(&example_link_tag(), example_link().tag(),);
    }

    #[test]
    fn link_entry_smoke_test() {
        test_link_entry();
    }

    #[test]
    fn link_entry_action_kind_test() {
        assert_eq!(&example_link_action_kind(), test_link_entry().action_kind(),);
    }

    #[test]
    fn link_entry_link_test() {
        assert_eq!(&example_link(), test_link_entry().link(),);
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
            LinkAddEntry::try_from(test_link_entry_json_string()).unwrap(),
            test_link_entry(),
        );
    }

    #[test]
    /// show ToEntry implementation for Link
    fn link_entry_to_entry_test() {
        // to_entry()
        assert_eq!(
            Entry::new(EntryType::LinkAdd, test_link_entry_json_string()),
            test_link_entry().to_entry(),
        );

        // from_entry()
        assert_eq!(
            test_link_entry(),
            LinkAddEntry::from_entry(&test_link_entry().to_entry()),
        );
    }
}
