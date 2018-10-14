use entry::{entry_type::EntryType, link::Link, Entry, ToEntry};
use serde_json;
use std::string::ToString;

//-------------------------------------------------------------------------------------------------
// LinkListEntry
//-------------------------------------------------------------------------------------------------
//
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
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

impl ToString for LinkListEntry {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("LinkListEntry failed to serialize")
    }
}

impl From<String> for LinkListEntry {
    fn from(s: String) -> LinkListEntry {
        serde_json::from_str(&s).expect("LinkListEntry failed to deserialize")
    }
}

impl ToEntry for LinkListEntry {
    // Convert a LinkListEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        Entry::new(&EntryType::LinkList, &self.to_string())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::LinkList, entry.entry_type());
        LinkListEntry::from(entry.value().to_owned())
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
