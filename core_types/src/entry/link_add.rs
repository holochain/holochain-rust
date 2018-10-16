use cas::content::Address;
use entry::Entry;
use entry::test_app_entry_b;
use entry::test_app_entry_a;
use cas::content::AddressableContent;

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

pub fn test_link_tag() -> LinkTag {
    LinkTag::from("foo-tag")
}

pub fn test_link_add() -> LinkAdd {
    LinkAdd::new(
        &test_app_entry_a().address(),
        &test_app_entry_b().address(),
        &test_link_tag(),
    )
}

pub fn test_link_add_entry() -> Entry {
    Entry::LinkAdd(test_link_add())
}

#[cfg(test)]
pub mod tests {

    use cas::content::AddressableContent;
    use entry::{
        test_app_entry_a, test_app_entry_b,
    };
    use entry::link_add::test_link_add;
    use entry::link_add::test_link_tag;

    #[test]
    fn link_smoke_test() {
        test_link_add();
    }

    #[test]
    fn link_base_test() {
        assert_eq!(&test_app_entry_a().address(), test_link_add().base(),);
    }

    #[test]
    fn link_target_test() {
        assert_eq!(&test_app_entry_b().address(), test_link_add().target(),);
    }

    #[test]
    fn link_tag_test() {
        assert_eq!(&test_link_tag(), test_link_add().tag(),);
    }
}
