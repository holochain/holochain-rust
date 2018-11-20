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

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct AppEntryType(String);

impl From<&'static str> for AppEntryType {
    fn from (s: &str) -> Self {
        AppEntryType(s.to_string())
    }
}

impl From<AppEntryType> for String {
    fn from (app_entry_type: AppEntryType) -> Self {
        app_entry_type.0.clone()
    }
}

impl ToString for AppEntryType {
    fn to_string(&self) -> String {
        String::from(self.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum SystemEntryType {
    Dna,
    AgentId,
    Delete,
    LinkAdd,
    LinkRemove,
    LinkList,
    ChainHeader,
    ChainMigrate,
}

// Enum for listing all System Entry Types
// Variant `Data` is for user defined entry types
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum EntryType {
    App(AppEntryType),
    System(SystemEntryType),
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
        match self {
            EntryType::System(SystemEntryType::Dna) => false,
            _ => true,
        }
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
        Ok(match s {
            sys_prefix!("agent_id") => EntryType::System(SystemEntryType::AgentId),
            sys_prefix!("delete") => EntryType::System(SystemEntryType::Delete),
            sys_prefix!("dna") => EntryType::System(SystemEntryType::Dna),
            sys_prefix!("chain_header") => EntryType::System(SystemEntryType::ChainHeader),
            sys_prefix!("link_add") => EntryType::System(SystemEntryType::LinkAdd),
            sys_prefix!("link_remove") => EntryType::System(SystemEntryType::LinkRemove),
            sys_prefix!("link_list") => EntryType::System(SystemEntryType::LinkList),
            sys_prefix!("chain_migrate") => EntryType::System(SystemEntryType::ChainMigrate),
            _ => EntryType::App(AppEntryType(s.into())),
        })
    }
}

impl From<EntryType> for String {
    fn from(entry_type: EntryType) -> String {
        String::from(match entry_type {
            EntryType::App(ref app_entry_type) => &app_entry_type.0,
            EntryType::System(system_entry_type) => match system_entry_type {
                SystemEntryType::AgentId => sys_prefix!("agent_id"),
                SystemEntryType::Delete => sys_prefix!("delete"),
                SystemEntryType::Dna => sys_prefix!("dna"),
                SystemEntryType::ChainHeader => sys_prefix!("chain_header"),
                SystemEntryType::LinkAdd => sys_prefix!("link_add"),
                SystemEntryType::LinkRemove => sys_prefix!("link_remove"),
                SystemEntryType::LinkList => sys_prefix!("link_list"),
                SystemEntryType::ChainMigrate => sys_prefix!("chain_migrate"),
            },
        })
    }
}

impl From<String> for EntryType {
    fn from(s: String) -> EntryType {
        EntryType::from_str(&s).expect("could not convert String to EntryType")
    }
}

impl From<&'static str> for EntryType {
    fn from(s: &str) -> EntryType {
        EntryType::from(String::from(s))
    }
}

impl Display for EntryType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", String::from(self.to_owned()))
    }
}

/// dummy entry type
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_type() -> AppEntryType {
    AppEntryType::from("testEntryType")
}

pub fn test_entry_type() -> EntryType {
    EntryType::App(test_app_entry_type())
}

/// dummy entry type, same as test_type()
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_type_a() -> AppEntryType {
    test_app_entry_type()
}

pub fn test_entry_type_a() -> EntryType {
    EntryType::App(test_app_entry_type_a())
}

/// dummy entry type, differs from test_type()
#[cfg_attr(tarpaulin, skip)]
pub fn test_app_entry_type_b() -> AppEntryType {
    AppEntryType::from("testEntryTypeB")
}

pub fn test_entry_type_b() -> EntryType {
    EntryType::App(test_app_entry_type_b())
}

// #[cfg_attr(tarpaulin, skip)]
// pub fn test_unpublishable_entry_type() -> EntryType {
//     EntryType::Dna
// }

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn test_types() -> Vec<EntryType> {
        vec![
            EntryType::App(AppEntryType::from("foo")),
            EntryType::System(SystemEntryType::Dna),
            EntryType::System(SystemEntryType::AgentId),
            EntryType::System(SystemEntryType::Delete),
            EntryType::System(SystemEntryType::LinkAdd),
            EntryType::System(SystemEntryType::LinkRemove),
            EntryType::System(SystemEntryType::LinkList),
            EntryType::System(SystemEntryType::ChainHeader),
            EntryType::System(SystemEntryType::ChainMigrate),
        ]
    }

    #[test]
    fn entry_type_kind() {
        assert!(EntryType::App(AppEntryType::from("")).is_app());
        assert!(!EntryType::App(AppEntryType::from("")).is_sys());
        assert!(EntryType::System(SystemEntryType::AgentId).is_sys());
        assert!(!EntryType::System(SystemEntryType::AgentId).is_app());
    }

    #[test]
    fn entry_type_valid_app_name() {
        assert!(EntryType::has_valid_app_name("agent_id"));
        assert!(!EntryType::has_valid_app_name("%agent_id"));
        assert!(!EntryType::has_valid_app_name(&String::from(
            EntryType::System(SystemEntryType::AgentId)
        )));
        assert!(!EntryType::has_valid_app_name(&String::new()));
        assert!(EntryType::has_valid_app_name("toto"));
        assert!(!EntryType::has_valid_app_name("%%"));
        // TODO #445 - do a real regex test in has_valid_app_name()
        // assert!(EntryType::has_valid_app_name("\n"));
    }

    #[test]
    fn entry_type_as_str_test() {
        for (type_str, variant) in vec![
            (sys_prefix!("dna"), EntryType::System(SystemEntryType::Dna)),
            (sys_prefix!("agent_id"), EntryType::System(SystemEntryType::AgentId)),
            (sys_prefix!("delete"), EntryType::System(SystemEntryType::Delete)),
            (sys_prefix!("link_add"), EntryType::System(SystemEntryType::LinkAdd)),
            (sys_prefix!("link_remove"), EntryType::System(SystemEntryType::LinkRemove)),
            (sys_prefix!("link_list"), EntryType::System(SystemEntryType::LinkList)),
            (sys_prefix!("chain_header"), EntryType::System(SystemEntryType::ChainHeader)),
            (sys_prefix!("chain_migrate"), EntryType::System(SystemEntryType::ChainMigrate)),
        ] {
            assert_eq!(
                variant,
                EntryType::from_str(type_str).expect("could not convert str to EntryType")
            );

            assert_eq!(type_str, &String::from(variant),);
        }
    }

    #[test]
    fn can_publish_test() {
        for t in test_types() {
            match t {
                EntryType::System(SystemEntryType::Dna) => assert!(!t.can_publish()),
                _ => assert!(t.can_publish()),
            }
        }
    }
}
