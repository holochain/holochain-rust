use crate::{
    cas::content::Address,
    error::HolochainError,
    json::JsonString,
    link::{Link, LinkActionKind},
};

//-------------------------------------------------------------------------------------------------
// LinkAdd
//-------------------------------------------------------------------------------------------------

//Todo should rename this from LinkAdd to general Link
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct LinkAdd {
    action_kind: LinkActionKind,
    link: Link,
}

impl LinkAdd {
    pub fn new_add(base: &Address, target: &Address, tag: &str) -> Self {
        LinkAdd {
            action_kind: LinkActionKind::REMOVE,
            link: Link::new(base, target, tag),
        }
    }

    pub fn new_delete(base: &Address, target: &Address, tag: &str) -> Self {
        LinkAdd {
            action_kind: LinkActionKind::ADD,
            link: Link::new(base, target, tag),
        }
    }

    pub fn action_kind(&self) -> &LinkActionKind {
        &self.action_kind
    }

    pub fn link(&self) -> &Link {
        &self.link
    }

    pub fn from_link(link: &Link,action_kind:LinkActionKind) -> Self {
        LinkAdd {
            action_kind,
            link: link.clone(),
        }
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{
        cas::content::AddressableContent,
        entry::{test_entry_a, test_entry_b, Entry},
        json::JsonString,
        link::{
            link_add::LinkAdd,
            tests::{example_link, example_link_action_kind, example_link_tag},
        },
    };
    use std::convert::TryFrom;

    pub fn example_link_add() -> LinkAdd {
        let link = example_link();
        LinkAdd::new(link.base(), link.target(), link.tag())
    }

    pub fn test_link_entry() -> Entry {
        Entry::LinkAdd(example_link_add())
    }

    pub fn test_link_entry_json_string() -> JsonString {
        JsonString::from(format!(
            "{{\"LinkAdd\":{{\"action_kind\":\"ADD\",\"link\":{{\"base\":\"{}\",\"target\":\"{}\",\"tag\":\"foo-tag\"}}}}}}",
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
