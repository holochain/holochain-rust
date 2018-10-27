use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

// Macro for statically concatanating the system entry prefix for entry types of system entries
macro_rules! sys_prefix {
    ($s:expr) => {
        concat!("%", $s)
    };
}

// Enum for listing all System Entry Types
// Variant `Data` is for user defined entry types
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum EntryType {
    AgentId,
    Deletion,
    App(String),
    Dna,
    ChainHeader,
    Key,
    Link,
    Migration,
    /// TODO #339 - This is different kind of SystemEntry for the DHT only.
    /// Should be moved into a different enum for DHT entry types.
    LinkList,
    AgentState,
}

impl EntryType {
    pub fn is_app(&self) -> bool {
        match self {
            EntryType::App(_) => true,
            _ => false,
        }
    }
    pub fn is_sys(&self) -> bool {
        !self.is_app()
    }

    pub fn can_publish(&self) -> bool {
        *self != EntryType::Dna
    }

    /// Checks entry_type_name is valid
    pub fn has_valid_app_name(entry_type_name: &str) -> bool {
        // TODO #445 - do a real regex test instead
        // must not be empty
        entry_type_name.len() > 0
        // Must not have sys_prefix
            && &entry_type_name[0..1] != "%"
    }
}

impl FromStr for EntryType {
    type Err = usize;
    // Note: Function always return Ok()
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            sys_prefix!("agent_id") => Ok(EntryType::AgentId),
            sys_prefix!("deletion") => Ok(EntryType::Deletion),
            sys_prefix!("dna") => Ok(EntryType::Dna),
            sys_prefix!("chain_header") => Ok(EntryType::ChainHeader),
            sys_prefix!("key") => Ok(EntryType::Key),
            sys_prefix!("link") => Ok(EntryType::Link),
            sys_prefix!("link_list") => Ok(EntryType::LinkList),
            sys_prefix!("migration") => Ok(EntryType::Migration),
            sys_prefix!("agent_state") => Ok(EntryType::AgentState),
            _ => Ok(EntryType::App(s.to_string())),
        }
    }
}

impl Display for EntryType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.as_str())
    }
}

impl EntryType {
    pub fn as_str(&self) -> &str {
        let ret = match *self {
            EntryType::App(ref s) => s,
            EntryType::AgentId => sys_prefix!("agent_id"),
            EntryType::Deletion => sys_prefix!("deletion"),
            EntryType::Dna => sys_prefix!("dna"),
            EntryType::ChainHeader => sys_prefix!("chain_header"),
            EntryType::Key => sys_prefix!("key"),
            EntryType::Link => sys_prefix!("link"),
            EntryType::LinkList => sys_prefix!("link_list"),
            EntryType::Migration => sys_prefix!("migration"),
            EntryType::AgentState => sys_prefix!("agent_state"),
        };
        ret
    }
}

/// dummy entry type
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_type() -> EntryType {
    EntryType::App(String::from("testEntryType"))
}

/// dummy entry type, same as test_type()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_type_a() -> EntryType {
    test_entry_type()
}

/// dummy entry type, differs from test_type()
#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_type_b() -> EntryType {
    EntryType::App(String::from("testEntryTypeB"))
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_sys_entry_type() -> EntryType {
    EntryType::AgentId
}

#[cfg_attr(tarpaulin, skip)]
pub fn test_unpublishable_entry_type() -> EntryType {
    EntryType::Dna
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn test_types() -> Vec<EntryType> {
        vec![
            EntryType::AgentId,
            EntryType::Deletion,
            EntryType::App(String::from("foo")),
            EntryType::Dna,
            EntryType::ChainHeader,
            EntryType::Key,
            EntryType::Link,
            EntryType::Migration,
            EntryType::LinkList,
        ]
    }

    #[test]
    fn entry_type_kind() {
        assert!(EntryType::App(String::new()).is_app());
        assert!(!EntryType::App(String::new()).is_sys());
        assert!(EntryType::AgentId.is_sys());
        assert!(!EntryType::AgentId.is_app());
    }

    #[test]
    fn entry_type_valid_app_name() {
        assert!(EntryType::has_valid_app_name("agent_id"));
        assert!(!EntryType::has_valid_app_name("%agent_id"));
        assert!(!EntryType::has_valid_app_name(EntryType::AgentId.as_str()));
        assert!(!EntryType::has_valid_app_name(&String::new()));
        assert!(EntryType::has_valid_app_name("toto"));
        assert!(!EntryType::has_valid_app_name("%%"));
        // TODO #445 - do a real regex test in has_valid_app_name()
        // assert!(EntryType::has_valid_app_name("\n"));
    }

    #[test]
    fn entry_type_as_str() {
        for (type_str, variant) in vec![
            (sys_prefix!("agent_id"), EntryType::AgentId),
            (sys_prefix!("deletion"), EntryType::Deletion),
            (sys_prefix!("dna"), EntryType::Dna),
            (sys_prefix!("chain_header"), EntryType::ChainHeader),
            (sys_prefix!("key"), EntryType::Key),
            (sys_prefix!("link"), EntryType::Link),
            (sys_prefix!("migration"), EntryType::Migration),
        ] {
            assert_eq!(
                variant,
                EntryType::from_str(type_str).expect("could not convert str to EntryType")
            );

            assert_eq!(type_str, variant.as_str(),);
        }
    }

    #[test]
    fn can_publish_test() {
        for t in test_types() {
            match t {
                EntryType::Dna => assert!(!t.can_publish()),
                _ => assert!(t.can_publish()),
            }
        }
    }
}
