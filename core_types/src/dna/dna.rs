use crate::{
    cas::content::{AddressableContent, Content},
    dna::{
        bridges::Bridge,
        capabilities::{Capability, FnDeclaration},
        entry_types::EntryTypeDef,
        wasm, zome,
    },
    entry::entry_type::EntryType,
    error::{DnaError, HolochainError},
    json::JsonString,
};
use entry::entry_type::AppEntryType;
use multihash;
use serde_json::{self, Value};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    hash::{Hash, Hasher},
};

/// serde helper, provides a default empty object
fn empty_object() -> Value {
    json!({})
}

/// serde helper, provides a default newly generated v4 uuid
fn zero_uuid() -> String {
    String::from("00000000-0000-0000-0000-000000000000")
}

/// Represents the top-level holochain dna object.
#[derive(Serialize, Deserialize, Clone, Debug, DefaultJson)]
pub struct Dna {
    /// The top-level "name" of a holochain application.
    #[serde(default)]
    pub name: String,

    /// The top-level "description" of a holochain application.
    #[serde(default)]
    pub description: String,

    /// The semantic version of your holochain application.
    #[serde(default)]
    pub version: String,

    /// A unique identifier to distinguish your holochain application.
    #[serde(default = "zero_uuid")]
    pub uuid: String,

    /// Which version of the holochain dna spec does this represent?
    #[serde(default)]
    pub dna_spec_version: String,

    /// Any arbitrary application properties can be included in this object.
    #[serde(default = "empty_object")]
    pub properties: Value,

    /// An array of zomes associated with your holochain application.
    #[serde(default)]
    pub zomes: BTreeMap<String, zome::Zome>,
}

impl AddressableContent for Dna {
    fn content(&self) -> Content {
        Content::from(self.to_owned())
    }

    fn try_from_content(content: &Content) -> Result<Self, HolochainError> {
        Ok(Dna::try_from(content.to_owned())?)
    }
}

impl Default for Dna {
    /// Provide defaults for a dna object.
    fn default() -> Self {
        Dna {
            name: String::new(),
            description: String::new(),
            version: String::new(),
            uuid: zero_uuid(),
            dna_spec_version: String::from("2.0"),
            properties: empty_object(),
            zomes: BTreeMap::new(),
        }
    }
}

impl Dna {
    /// Create a new in-memory dna structure with some default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use holochain_core_types::dna::Dna;
    ///
    /// let dna = Dna::new();
    /// assert_eq!("", dna.name);
    ///
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Generate a pretty-printed json string from an in-memory dna struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use holochain_core_types::dna::Dna;
    ///
    /// let dna = Dna::new();
    /// println!("json: {}", dna.to_json_pretty().expect("DNA should serialize"));
    ///
    /// ```
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Return a Zome
    pub fn get_zome(&self, zome_name: &str) -> Option<&zome::Zome> {
        self.zomes.get(zome_name)
    }

    /// Return a Zome's Capability from a Zome and a Capability name.
    pub fn get_capability<'a>(
        &'a self,
        zome: &'a zome::Zome,
        capability_name: &str,
    ) -> Option<&'a Capability> {
        zome.capabilities.get(capability_name)
    }

    /// Return a Function declaration from a Zome
    pub fn get_function<'a>(
        &'a self,
        zome: &'a zome::Zome,
        function_name: &str,
    ) -> Option<&'a FnDeclaration> {
        zome.fn_declarations
            .iter()
            .find(|ref fn_decl| fn_decl.name == function_name)
    }

    /// Return a Zome Function declaration from a Zome name and Function name.
    pub fn get_function_with_zome_name(
        &self,
        zome_name: &str,
        fn_name: &str,
    ) -> Result<&FnDeclaration, DnaError> {
        // Zome must exist in DNA
        let zome = self.get_zome(zome_name);
        if zome.is_none() {
            return Err(DnaError::ZomeNotFound(format!(
                "Zome '{}' not found",
                &zome_name,
            )));
        }
        let zome = zome.unwrap();
        // Function must exist in Zome
        let fn_decl = self.get_function(zome, &fn_name);
        if fn_decl.is_none() {
            return Err(DnaError::ZomeFunctionNotFound(format!(
                "Zome function '{}' not found in Zome '{}'",
                &fn_name, &zome_name
            )));
        }
        // Everything OK
        Ok(fn_decl.unwrap())
    }

    /// Find a Zome and return it's WASM bytecode for a specified Capability
    pub fn get_wasm_from_zome_name<T: Into<String>>(&self, zome_name: T) -> Option<&wasm::DnaWasm> {
        let zome_name = zome_name.into();
        let zome = self.get_zome(&zome_name)?;
        Some(&zome.code)
    }

    /// Return a Zome's Capability from a Zome name and Capability name.
    pub fn get_capability_with_zome_name(
        &self,
        zome_name: &str,
        cap_name: &str,
    ) -> Result<&Capability, DnaError> {
        // Zome must exist in DNA
        let zome = self.get_zome(zome_name);
        if zome.is_none() {
            return Err(DnaError::ZomeNotFound(format!(
                "Zome '{}' not found",
                &zome_name,
            )));
        }
        let zome = zome.unwrap();
        // Capability must exist in Zome
        let cap = self.get_capability(zome, &cap_name);
        if cap.is_none() {
            return Err(DnaError::CapabilityNotFound(format!(
                "Capability '{}' not found in Zome '{}'",
                &cap_name, &zome_name
            )));
        }
        // Everything OK
        Ok(cap.unwrap())
    }

    /// Return the name of the zome holding a specified app entry_type
    pub fn get_zome_name_for_app_entry_type(
        &self,
        app_entry_type: &AppEntryType,
    ) -> Option<String> {
        let entry_type_name = String::from(app_entry_type.to_owned());
        // pre-condition: must be a valid app entry_type name
        assert!(EntryType::has_valid_app_name(&entry_type_name));
        // Browse through the zomes
        for (zome_name, zome) in &self.zomes {
            for (zome_entry_type_name, _) in &zome.entry_types {
                if *zome_entry_type_name
                    == EntryType::App(AppEntryType::from(entry_type_name.to_string()))
                {
                    return Some(zome_name.clone());
                }
            }
        }
        None
    }

    /// Return the entry_type definition of a specified app entry_type
    pub fn get_entry_type_def(&self, entry_type_name: &str) -> Option<&EntryTypeDef> {
        // pre-condition: must be a valid app entry_type name
        assert!(EntryType::has_valid_app_name(entry_type_name));
        // Browse through the zomes
        for (_zome_name, zome) in &self.zomes {
            for (zome_entry_type_name, entry_type_def) in &zome.entry_types {
                if *zome_entry_type_name
                    == EntryType::App(AppEntryType::from(entry_type_name.to_string()))
                {
                    return Some(entry_type_def);
                }
            }
        }
        None
    }

    pub fn multihash(&self) -> Result<Vec<u8>, HolochainError> {
        let s = String::from(JsonString::from(self.to_owned()));
        multihash::encode(multihash::Hash::SHA2256, &s.into_bytes())
            .map_err(|error| HolochainError::ErrorGeneric(error.to_string()))
    }

    pub fn get_required_bridges(&self) -> Vec<Bridge> {
        self.zomes
            .values()
            .map(|zome| zome.get_required_bridges())
            .flatten()
            .collect()
    }
}

impl Hash for Dna {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let s = String::from(JsonString::from(self.to_owned()));
        s.hash(state);
    }
}

impl PartialEq for Dna {
    fn eq(&self, other: &Dna) -> bool {
        // need to guarantee that PartialEq and Hash always agree
        JsonString::from(self.to_owned()) == JsonString::from(other.to_owned())
    }
}
