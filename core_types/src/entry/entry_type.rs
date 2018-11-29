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
    App(String),
    Dna,
    AgentId,
    Delete,
    LinkAdd,
    LinkRemove,
    LinkList,
    ChainHeader,
    ChainMigrate,
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
        /*
                let dna = context
            .state()
            .expect("context must have a State.")
            .nucleus()
            .dna()
            .expect("context.state must hold DNA in order to commit an app entry.");
        let maybe_def = dna.get_entry_type_def(&entry.entry_type().to_string());
        if maybe_def.is_none() {
            // TODO #439 - Log the error. Once we have better logging.
            return None;
        }
        let entry_type_def = maybe_def.unwrap();

        // app entry type must be publishable
        if !entry_type_def.sharing.clone().can_publish() {
            return None;
        }
            */
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
            sys_prefix!("delete") => Ok(EntryType::Delete),
            sys_prefix!("dna") => Ok(EntryType::Dna),
            sys_prefix!("chain_header") => Ok(EntryType::ChainHeader),
            sys_prefix!("link_add") => Ok(EntryType::LinkAdd),
            sys_prefix!("link_remove") => Ok(EntryType::LinkRemove),
            sys_prefix!("link_list") => Ok(EntryType::LinkList),
            sys_prefix!("chain_migrate") => Ok(EntryType::ChainMigrate),
            _ => Ok(EntryType::App(s.to_string())),
        }
    }
}

impl From<EntryType> for String {
    fn from(entry_type: EntryType) -> String {
        String::from(match entry_type {
            EntryType::App(ref s) => s,
            EntryType::AgentId => sys_prefix!("agent_id"),
            EntryType::Delete => sys_prefix!("delete"),
            EntryType::Dna => sys_prefix!("dna"),
            EntryType::ChainHeader => sys_prefix!("chain_header"),
            EntryType::LinkAdd => sys_prefix!("link_add"),
            EntryType::LinkRemove => sys_prefix!("link_remove"),
            EntryType::LinkList => sys_prefix!("link_list"),
            EntryType::ChainMigrate => sys_prefix!("chain_migrate"),
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
            EntryType::App(String::from("foo")),
            EntryType::Dna,
            EntryType::AgentId,
            EntryType::Delete,
            EntryType::LinkAdd,
            EntryType::LinkRemove,
            EntryType::LinkList,
            EntryType::ChainHeader,
            EntryType::ChainMigrate,
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
        assert!(!EntryType::has_valid_app_name(&String::from(
            EntryType::AgentId
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
            (sys_prefix!("dna"), EntryType::Dna),
            (sys_prefix!("agent_id"), EntryType::AgentId),
            (sys_prefix!("delete"), EntryType::Delete),
            (sys_prefix!("link_add"), EntryType::LinkAdd),
            (sys_prefix!("link_remove"), EntryType::LinkRemove),
            (sys_prefix!("link_list"), EntryType::LinkList),
            (sys_prefix!("chain_header"), EntryType::ChainHeader),
            (sys_prefix!("chain_migrate"), EntryType::ChainMigrate),
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
                EntryType::Dna => assert!(!t.can_publish()),
                _ => assert!(t.can_publish()),
            }
        }
    }
}
