use crate::link::{Link, LinkActionKind};

use lib3h_persistence_api::{cas::content::Address, error::PersistenceError, json::JsonString};

//-------------------------------------------------------------------------------------------------
// LinkData
//-------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct LinkData {
    pub action_kind: LinkActionKind,
    pub link: Link,
}

impl LinkData {
    pub fn new_add(base: &Address, target: &Address, link_type: &str, tag: &str) -> Self {
        LinkData {
            action_kind: LinkActionKind::ADD,
            link: Link::new(base, target, link_type, tag),
        }
    }

    pub fn new_delete(base: &Address, target: &Address, link_type: &str, tag: &str) -> Self {
        LinkData {
            action_kind: LinkActionKind::REMOVE,
            link: Link::new(base, target, link_type, tag),
        }
    }

    pub fn action_kind(&self) -> &LinkActionKind {
        &self.action_kind
    }

    pub fn link(&self) -> &Link {
        &self.link
    }

    pub fn from_link(link: &Link, action_kind: LinkActionKind) -> Self {
        LinkData {
            action_kind,
            link: link.clone(),
        }
    }

    pub fn add_from_link(link: &Link) -> Self {
        Self::from_link(link, LinkActionKind::ADD)
    }

    pub fn remove_from_link(link: &Link) -> Self {
        Self::from_link(link, LinkActionKind::REMOVE)
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{
        entry::{test_entry_a, test_entry_b, Entry},
        link::{
            link_data::LinkData,
            tests::{example_link, example_link_action_kind, example_link_type},
        },
    };
    use lib3h_persistence_api::{cas::content::AddressableContent, json::JsonString};
    use std::convert::TryFrom;

    pub fn example_link_add() -> LinkData {
        let link = example_link();
        LinkData::new_add(link.base(), link.target(), link.link_type(), link.tag())
    }

    pub fn test_link_entry() -> Entry {
        Entry::LinkAdd(example_link_add())
    }

    pub fn test_link_entry_json_string() -> JsonString {
        JsonString::from_json(&format!(
            "{{\"LinkAdd\":{{\"action_kind\":\"ADD\",\"link\":{{\"base\":\"{}\",\"target\":\"{}\",\"link_type\":\"foo-link-type\",\"tag\":\"foo-link-tag\"}}}}}}",
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
    fn link_type_test() {
        assert_eq!(&example_link_type(), example_link().link_type(),);
    }

    #[test]
    fn link_entry_smoke_test() {
        test_link_entry();
    }

    #[test]
    fn link_add_action_kind_test() {
        assert_eq!(
            &example_link_action_kind(),
            example_link_add().action_kind(),
        );
    }

    #[test]
    fn link_add_link_test() {
        assert_eq!(&example_link(), example_link_add().link(),);
    }

    #[test]
    /// show ToString for LinkAdd
    fn link_entry_to_string_test() {
        assert_eq!(
            test_link_entry_json_string(),
            JsonString::from(test_link_entry()),
        );
    }

    #[test]
    /// show From<String> for LinkAdd
    fn link_entry_from_string_test() {
        assert_eq!(
            Entry::try_from(test_link_entry_json_string()).unwrap(),
            test_link_entry(),
        );
    }
}
