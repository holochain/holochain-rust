//! holochain_core_types::dna::zome is a set of structs for working with holochain dna.

use crate::{
    dna::{
        bridges::{Bridge, BridgePresence},
        fn_declarations::{FnDeclaration, FnParameter, TraitFns},
        traits::ReservedTraitNames,
        wasm::DnaWasm,
    },
    entry::entry_type::EntryType,
};

use holochain_json_api::{error::JsonError, json::JsonString};

use dna::entry_types::{self, deserialize_entry_types, serialize_entry_types, EntryTypeDef};
use std::collections::BTreeMap;

/// Represents the "config" object on a "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct Config {}

impl Default for Config {
    /// Provide defaults for the "zome" "config" object.
    fn default() -> Self {
        Config {}
    }
}

impl Config {
    /// Allow sane defaults for `Config::new()`.
    pub fn new() -> Self {
        Default::default()
    }
}

pub type ZomeEntryTypes = BTreeMap<EntryType, EntryTypeDef>;
pub type ZomeTraits = BTreeMap<String, TraitFns>;
pub type ZomeFnDeclarations = Vec<FnDeclaration>;

/// Represents an individual "zome".
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct Zome {
    /// A description of this zome.
    #[serde(default)]
    pub description: String,

    /// Configuration associated with this zome.
    /// Note, this should perhaps be a more free-form serde_json::Value,
    /// "throw-errors" may not make sense for wasm, or other ribosome types.
    #[serde(default)]
    pub config: Config,

    /// An array of entry_types associated with this zome.
    #[serde(default)]
    #[serde(serialize_with = "serialize_entry_types")]
    #[serde(deserialize_with = "deserialize_entry_types")]
    pub entry_types: ZomeEntryTypes,

    /// An array of traits defined in this zome.
    #[serde(default)]
    pub traits: ZomeTraits,

    /// An array of functions declared in this this zome.
    #[serde(default)]
    pub fn_declarations: ZomeFnDeclarations,

    /// Validation code for this entry_type.
    pub code: DnaWasm,

    /// A list of bridges to other DNAs that this DNA can use or depends on.
    #[serde(default)]
    pub bridges: Vec<Bridge>,
}

impl Eq for Zome {}

impl Zome {
    /// Provide defaults for an individual "zome".
    pub fn empty() -> Self {
        Zome {
            description: String::new(),
            config: Config::new(),
            entry_types: BTreeMap::new(),
            fn_declarations: Vec::new(),
            traits: BTreeMap::new(),
            code: DnaWasm::new_invalid(),
            bridges: Vec::new(),
        }
    }

    /// Allow sane defaults for `Zome::new()`.
    pub fn new(
        description: &str,
        config: &Config,
        entry_types: &BTreeMap<EntryType, entry_types::EntryTypeDef>,
        fn_declarations: &[FnDeclaration],
        traits: &BTreeMap<String, TraitFns>,
        code: &DnaWasm,
    ) -> Zome {
        Zome {
            description: description.into(),
            config: config.clone(),
            entry_types: entry_types.to_owned(),
            fn_declarations: fn_declarations.to_owned(),
            traits: traits.to_owned(),
            code: code.clone(),
            bridges: Vec::new(),
        }
    }

    pub fn get_required_bridges(&self) -> Vec<Bridge> {
        self.bridges
            .iter()
            .filter(|bridge| bridge.presence == BridgePresence::Required)
            .cloned()
            .collect()
    }

    /// Add a function declaration to a Zome
    pub fn add_fn_declaration(
        &mut self,
        name: String,
        inputs: Vec<FnParameter>,
        outputs: Vec<FnParameter>,
    ) {
        self.fn_declarations.push(FnDeclaration {
            name,
            inputs,
            outputs,
        });
    }

    /// Return a Function declaration from a Zome
    pub fn get_function(&self, fn_name: &str) -> Option<&FnDeclaration> {
        self.fn_declarations
            .iter()
            .find(|ref fn_decl| fn_decl.name == fn_name)
    }

    // Helper function for finding out if a given function call is public
    pub fn is_fn_public(&self, fn_name: &str) -> bool {
        let pub_trait = ReservedTraitNames::Public.as_str();
        self.traits.iter().any(|(trait_name, trait_fns)| {
            trait_name == pub_trait && trait_fns.functions.contains(&fn_name.to_owned())
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::dna::{
        fn_declarations::FnParameter,
        zome::{entry_types::EntryTypeDef, Zome},
    };
    use serde_json;
    use std::{collections::BTreeMap, convert::TryFrom, sync::Arc};

    pub fn test_zome() -> Zome {
        Zome::empty()
    }

    #[test]
    fn build_and_compare() {
        let fixture: Zome = serde_json::from_str(
            r#"{
                "description": "test",
                "config": {},
                "entry_types": {},
                "fn_delcarations": [],
                "traits": {},
                "code": {
                    "code": ""
                }
            }"#,
        )
        .unwrap();

        let mut zome = Zome::empty();
        zome.description = String::from("test");

        assert_eq!(fixture, zome);
    }

    #[test]
    fn zome_json_test() {
        let mut entry_types = BTreeMap::new();
        entry_types.insert(EntryType::from("foo"), EntryTypeDef::new());
        let mut zome = Zome::empty();
        zome.entry_types = entry_types;

        let expected = "{\"description\":\"\",\"config\":{},\"entry_types\":{\"foo\":{\"properties\":\"{}\",\"sharing\":\"public\",\"links_to\":[],\"linked_from\":[]}},\"traits\":{},\"fn_declarations\":[],\"code\":{\"code\":\"\"},\"bridges\":[]}";

        assert_eq!(
            JsonString::from_json(expected),
            JsonString::from(zome.clone()),
        );

        assert_eq!(
            zome,
            Zome::try_from(JsonString::from_json(expected)).unwrap(),
        );

        // Ensure that compressed large WASM gets round-tripped to [String, ...]
        zome.code.code = Arc::new(vec![
                0x40, 0x35, 0xe0, 0x39, 0xa6, 0x27, 0x8a, 0x62, 0x9f, 0xa8, 0x2b, 0x2b, 0x7e, 0xbb, 0xed, 0xdb,
                0xb0, 0x90, 0x01, 0xf4, 0x07, 0xa6, 0xe2, 0xd5, 0xd1, 0x30, 0x1f, 0xe8, 0x77, 0x1b, 0xf0, 0x08,
                0x14, 0x0b, 0x09, 0xf3, 0xfc, 0xec, 0x1f, 0x76, 0x3e, 0xcd, 0x94, 0xe0, 0xe3, 0xbc, 0xe6, 0xe4,
                0x36, 0xd1, 0x7e, 0x77, 0xd7, 0xc4, 0x89, 0x8d, 0x6f, 0x78, 0x9d, 0x0b, 0xba, 0xa4, 0x53, 0xc7,
                0x37, 0x76, 0xd1, 0xf8, 0x5e, 0x62, 0x29, 0xc2, 0x2d, 0x3f, 0x9a, 0x8d, 0xb0, 0x1f, 0xe4, 0x72,
                0xaf, 0x34, 0x1f, 0x60, 0xb1, 0x24, 0x88, 0x8c, 0x33, 0x0d, 0xd6, 0xd3, 0x5c, 0xb3, 0xa7, 0xcc,
                0xd3, 0x62, 0xb2, 0x67, 0x17, 0x5d, 0xe6, 0x7a, 0x99, 0x12, 0x2c, 0x06, 0xa9, 0xce, 0xbe, 0xa9,
                0xbb, 0xa3, 0x89, 0x26, 0x45, 0x1a, 0xcf, 0xef, 0x69, 0xd0, 0x22, 0x4a, 0x8f, 0xd6, 0x07, 0xed,
                0x24, 0xca, 0xfe, 0xbb, 0x7e, 0x31, 0x5f, 0xe0, 0x7d, 0x5e, 0x59, 0x40, 0x21, 0x5f, 0x29, 0xfb,
                0x6f, 0x8f, 0xe5, 0x3b, 0xaa, 0xe4, 0x45, 0x51, 0xa0, 0x37, 0x0a, 0x20, 0xc0, 0x93, 0xb5, 0x46,
                0x7f, 0xf3, 0x48, 0x20, 0x69, 0xcb, 0x46, 0x0a, 0x10, 0xa1, 0x49, 0xed, 0xc8, 0x60, 0xa4, 0xc9,
                0xe9, 0x91, 0x15, 0x2a, 0xf0, 0xdc, 0x38, 0x2a, 0x5e, 0xe1, 0xb8, 0xa9, 0xad, 0x31, 0x96, 0x98,
                0xc3, 0xe0, 0x74, 0x61, 0xe9, 0x15, 0xb8, 0x6c, 0x4d, 0x37, 0x9c, 0xcc, 0xb6, 0x15, 0x32, 0xb3,
                0x0c, 0x30, 0x8d, 0x15, 0xd6, 0x6e, 0x0b, 0x68, 0x80, 0x9c, 0x0d, 0xb5, 0xcd, 0xdc, 0x86, 0xcc,
                0x98, 0xed, 0x1d, 0xc1, 0x67, 0x31, 0x80, 0x7a, 0x17, 0x26, 0x7e, 0x57, 0x0e, 0xd4, 0x52, 0xc1,
                0xac, 0x29, 0x60, 0x36, 0x28, 0xe9, 0x5b, 0x3e, 0x87, 0x81, 0x46, 0xe5, 0x78, 0xa9, 0xdb, 0x42,
                0xcc, 0x2a, 0x70, 0x8a, 0x8e, 0x91, 0xe3, 0xbf, 0x48, 0x77, 0x66, 0x52, 0x46, 0x41, 0x8b, 0xc0,
                0x83, 0x9a, 0x43, 0xfb, 0xdf, 0xeb, 0x34, 0x10, 0x0c, 0x3f, 0x42, 0x0c, 0xa6, 0xec, 0x27, 0xff,
                0xb0, 0xb4, 0x5c, 0x17, 0x8d, 0x91, 0x2f, 0x07, 0x56, 0x78, 0xfa, 0x0a, 0xfb, 0x27, 0x83, 0x65,
                0xb5, 0xe9, 0x8c, 0xe3, 0x2d, 0x08, 0x34, 0xde, 0xcc, 0x51, 0x45, 0xb3, 0x47, 0x4b, 0x05, 0xbb,
                0xaf, 0xcb, 0xac, 0x62, 0x73, 0xd7, 0x0e, 0x78, 0x57, 0xc5, 0x8d, 0xbe, 0x91, 0x23, 0xe0, 0xce,
                0xeb, 0x6b, 0x54, 0x72, 0x77, 0x45, 0x10, 0xec, 0xfb, 0x08, 0x42, 0x77, 0x56, 0xa4, 0x5e, 0x50,
                0x6f, 0x77, 0xc5, 0xa8, 0x4f, 0x2f, 0xfd, 0x5a, 0xa5, 0x06, 0xdc, 0xdc, 0x17, 0xb3, 0x33, 0x1c,
                0x94, 0x86, 0x2e, 0xca, 0x28, 0x50, 0x49, 0xb9, 0x17, 0x9a, 0x01, 0x2d, 0xc8, 0x70, 0x0a, 0x79,
                0x24, 0xba, 0x7c, 0x65, 0x52, 0x85, 0x3a, 0x17, 0x91, 0x95, 0xb2, 0xb9, 0xbe, 0x90, 0xcb, 0x95,
                0x71, 0x14, 0xc6, 0xb2, 0x50, 0xe8, 0xe9, 0x33, 0x77, 0x4f, 0xea, 0x7e, 0x27, 0x3f, 0x16, 0x08,
                0x07, 0x3f, 0x44, 0xce, 0xbf, 0x64, 0x06, 0xf9, 0xa3, 0x6b, 0x32, 0x36, 0x5f, 0x84, 0x23, 0x13,
                0x07, 0x55, 0x79, 0x5f, 0xb6, 0x67, 0x24, 0x04, 0x53, 0x1f, 0x44, 0xeb, 0xa9, 0xd7, 0xad, 0xe1,
                0x1a, 0x0f, 0xb7, 0x55, 0x54, 0x03, 0xad, 0x76, 0xfd, 0xc6, 0xf6, 0x8a, 0x84, 0xb1, 0x3f, 0xd4,
                0x1b, 0x52, 0x3e, 0xdc, 0xd9, 0x65, 0x95, 0x07, 0xd6, 0xfd, 0xc4, 0x97, 0x82, 0x63, 0x64, 0xcd,
                0xd6, 0xc7, 0xd9, 0x25, 0xa8, 0x1f, 0x72, 0x04, 0xdc, 0x51, 0xd3, 0x9c, 0x89, 0x66, 0x63, 0xa9,
                0xd3, 0x60, 0xaa, 0x50, 0xfa, 0x2f, 0xaf, 0x52, 0xc7, 0xa1, 0x8f, 0x81, 0x28, 0x6a, 0x8f, 0xe5,
        ]);
        /*
        zome.code = Arc::new(hex!("
            40 35 e0 39 a6 27 8a 62 9f a8 2b 2b 7e bb ed db
            b0 90 01 f4 07 a6 e2 d5 d1 30 1f e8 77 1b f0 08
            14 0b 09 f3 fc ec 1f 76 3e cd 94 e0 e3 bc e6 e4
            36 d1 7e 77 d7 c4 89 8d 6f 78 9d 0b ba a4 53 c7
            37 76 d1 f8 5e 62 29 c2 2d 3f 9a 8d b0 1f e4 72
            af 34 1f 60 b1 24 88 8c 33 0d d6 d3 5c b3 a7 cc
            d3 62 b2 67 17 5d e6 7a 99 12 2c 06 a9 ce be a9
            bb a3 89 26 45 1a cf ef 69 d0 22 4a 8f d6 07 ed
            24 ca fe bb 7e 31 5f e0 7d 5e 59 40 21 5f 29 fb
            6f 8f e5 3b aa e4 45 51 a0 37 0a 20 c0 93 b5 46
            7f f3 48 20 69 cb 46 0a 10 a1 49 ed c8 60 a4 c9
            e9 91 15 2a f0 dc 38 2a 5e e1 b8 a9 ad 31 96 98
            c3 e0 74 61 e9 15 b8 6c 4d 37 9c cc b6 15 32 b3
            0c 30 8d 15 d6 6e 0b 68 80 9c 0d b5 cd dc 86 cc
            98 ed 1d c1 67 31 80 7a 17 26 7e 57 0e d4 52 c1
            ac 29 60 36 28 e9 5b 3e 87 81 46 e5 78 a9 db 42
            cc 2a 70 8a 8e 91 e3 bf 48 77 66 52 46 41 8b c0
            83 9a 43 fb df eb 34 10 0c 3f 42 0c a6 ec 27 ff
            b0 b4 5c 17 8d 91 2f 07 56 78 fa 0a fb 27 83 65
            b5 e9 8c e3 2d 08 34 de cc 51 45 b3 47 4b 05 bb
            af cb ac 62 73 d7 0e 78 57 c5 8d be 91 23 e0 ce
            eb 6b 54 72 77 45 10 ec fb 08 42 77 56 a4 5e 50
            6f 77 c5 a8 4f 2f fd 5a a5 06 dc dc 17 b3 33 1c
            94 86 2e ca 28 50 49 b9 17 9a 01 2d c8 70 0a 79
            24 ba 7c 65 52 85 3a 17 91 95 b2 b9 be 90 cb 95
            71 14 c6 b2 50 e8 e9 33 77 4f ea 7e 27 3f 16 08
            07 3f 44 ce bf 64 06 f9 a3 6b 32 36 5f 84 23 13
            07 55 79 5f b6 67 24 04 53 1f 44 eb a9 d7 ad e1
            1a 0f b7 55 54 03 ad 76 fd c6 f6 8a 84 b1 3f d4
            1b 52 3e dc d9 65 95 07 d6 fd c4 97 82 63 64 cd
            d6 c7 d9 25 a8 1f 72 04 dc 51 d3 9c 89 66 63 a9
            d3 60 aa 50 fa 2f af 52 c7 a1 8f 81 28 6a 8f e5
            8d 95 f4 4d 27 45 07 9c f7 26 8d 41 45 28 72 5d
            7b 90 6f 8b 81 84 c6 47 2a e5 91 c3 9a 84 81 52
            cc c3 b7 6d 8f 94 59 d6 0d eb eb 80 08 52 fc 51
            fb e8 77 76 f2 d6 af c6 14 01 43 72 18 f3 9d bd
            36 ff c5 04 0f 91 25 82 2a c8 53 12 42 15 9a f7
            46 08 d4 34 22 a4 e4 29 73 d6 6e 86 57 60 e4 ae
            74 c9 13 ef c9 fd 25 07 1d 9d 26 44 07 94 03 73
            ab ee df a1 1b ad 19 1c 3a f2 1a 9b 65 ca 80 6a
            28 02 9c bd 41 d8 49 f1 79 7a 24 ab 28 03 2b 84
            93 81 33 34 89 00 0a 16 a2 ef fa e2 3a dc b0 21
            08 b1 01 0f 2c e4 4e e9 6e 36 09 76 11 4d e1 3c
            6a d4 87 0b e4 cf 51 f9 31 b8 0f 9c bb 4d cc 79
            5c de 8d 3b 04 17 9a dd 71 38 17 5b 63 70 46 58
            50 85 d8 8d 03 98 ad 1c d0 4a 18 ea 4b 89 25 06
            b8 e3 f7 3a 74 2b a2 7a 3a 8b 74 1c e3 eb 66 44
            84 d8 44 8e c3 22 18 7d f7 97 bc c0 5e d6 59 00
            7b ce c6 7c 36 d5 03 ee 92 b7 87 8a 76 46 42 59
            fb ec 18 7a fb dc e8 55 e5 7d 69 e2 3b e3 a6 be
            3a af 06 ec 56 25 14 bc 32 26 7b f6 9a d4 9d a0
            5e 7a 98 23 21 3b 2e ec c1 bb 17 81 5d d1 8c e9
            c8 44 04 8f 8a 91 37 36 d6 c2 a9 c0 b4 32 48 f3
            c6 d1 70 df 13 a0 3b b4 ee 40 66 5d c4 c5 77 7f
            1f 9a 53 57 f4 b3 d5 05 3d b7 ef 19 bf 13 85 e3
            af f2 c8 ae 95 4e a4 5d 69 7f 04 82 7b f7 c0 0f
            50 33 6b 77 0a b0 7f e9 96 ab 7c a5 10 17 d2 94
            36 dd cb 1d f9 1b 41 1d 6f c0 a7 58 a3 c2 42 40
            c4 83 cb c6 ef 5c 3d 71 2b c1 44 e9 85 ed 80 42
            d5 de 20 c9 2c 82 24 b6 52 93 b7 c7 12 8f 65 34
            43 87 08 23 0e 57 ca 19 c0 4e 1f d0 1a 7a 29 aa
            8f 81 c0 49 70 d6 55 5b bb bd 1e 69 23 08 ba 86
            38 2d 53 07 16 ef 8b 10 e1 13 a0 99 d7 15 26 6d
            63 65 41 23 df bb d4 0f b4 77 1b 8b d8 ff 49 64
            83 3b c8 38 b6 a5 a2 1b 7d 9e 59 36 de 2a ba ea
            6c f1 a6 e9 f0 92 31 99 32 04 5a 7f ea e9 74 7f
            17 d0 3e 2f a4 1e 85 19 3d 42 23 6d be be fc d1
            a5 18 87 bd 5f dc 94 d9 eb 9e 2b 9d a4 4d 78 87
            de c0 dc 9e 99 43 f6 83 8f a9 df 78 58 b3 c2 56
            c3 15 b6 ef 1a 00 23 7b 94 c6 48 8a 1e 17 18 a9
            10 dd 45 ce a2 e4 e0 76 f9 30 9f 41 a1 69 d3 57
            b8 3a f4 c1 43 03 32 ba 8e a2 98 31 5d 13 de eb
            4c d7 c9 62 31 87 8b 3b 7d 17 bd 78 bb 98 59 08
            de 94 c7 34 9c d1 0a 8b 03 36 b3 4f 9b e5 62 6e
            8f 0a 8a d5 38 22 d0 f0 41 cb 78 6e e2 95 2a f1
            fb f3 95 d7 a7 87 26 49 a3 f8 6d cc 96 7f 37 f9
            9f 38 91 6e 7f 32 24 37 8f 5e 9b e0 aa f9 fb c0
            29 01 e8 7c ee 03 64 46 80 1e 15 4b c7 55 60 3b
            7c 0b 13 2b 4b 06 ee eb fa 92 7c ff b1 38 5e 13
            ca 15 86 e3 dd 15 04 a6 df 69 3a 7c 0d 04 03 04
        "));
        */

        let expected_big_wasm = "{\"description\":\"\",\"config\":{},\"entry_types\":{\"foo\":{\"properties\":\"{}\",\"sharing\":\"public\",\"links_to\":[],\"linked_from\":[]}},\"traits\":{},\"fn_declarations\":[],\"code\":{\"code\":[\"H4sIAAAAAAAA/wEAAv/9QDXgOaYnimKfqCsrfrvt27CQAfQHpuLV0TAf6Hcb8AgUCwnz/Owfdj7NlODjvObkNtF+d9fEiY1veJ0LuqRTxzd20fheYinCLT+ajbAf5HKv\",\"NB9gsSSIjDMN1tNcs6fM02KyZxdd5nqZEiwGqc6+qbujiSZFGs/vadAiSo/WB+0kyv67fjFf4H1eWUAhXyn7b4/lO6rkRVGgNwogwJO1Rn/zSCBpy0YKEKFJ7chgpMnp\",\"kRUq8Nw4Kl7huKmtMZaYw+B0YekVuGxNN5zMthUyswwwjRXWbgtogJwNtc3chsyY7R3BZzGAehcmflcO1FLBrClgNijpWz6HgUbleKnbQswqcIqOkeO/SHdmUkZBi8CD\",\"mkP73+s0EAw/Qgym7Cf/sLRcF42RLwdWePoK+yeDZbXpjOMtCDTezFFFs0dLBbuvy6xic9cOeFfFjb6RI+DO62tUcndFEOz7CEJ3VqReUG93xahPL/1apQbc3BezMxyU\",\"hi7KKFBJuReaAS3IcAp5JLp8ZVKFOheRlbK5vpDLlXEUxrJQ6Okzd0/qfic/FggHP0TOv2QG+aNrMjZfhCMTB1V5X7ZnJARTH0Trqdet4RoPt1VUA612/cb2ioSxP9Qb\",\"Uj7c2WWVB9b9xJeCY2TN1sfZJagfcgTcUdOciWZjqdNgqlD6L69Sx6GPgShqj+WBTIVcAAIAAA==\"]},\"bridges\":[]}";

        assert_eq!(
            JsonString::from_json(expected_big_wasm),
            JsonString::from(zome.clone()),
        );
        assert_eq!(
            zome,
            Zome::try_from(JsonString::from_json(expected_big_wasm)).unwrap(),
        );
    }

    #[test]
    fn test_zome_add_fn_declaration() {
        let mut zome = Zome::empty();
        assert_eq!(zome.fn_declarations.len(), 0);
        zome.add_fn_declaration(
            String::from("hello"),
            vec![],
            vec![FnParameter {
                name: String::from("greeting"),
                parameter_type: String::from("String"),
            }],
        );
        assert_eq!(zome.fn_declarations.len(), 1);

        let expected = "[FnDeclaration { name: \"hello\", inputs: [], outputs: [FnParameter { parameter_type: \"String\", name: \"greeting\" }] }]";
        assert_eq!(expected, format!("{:?}", zome.fn_declarations),);
    }

    #[test]
    fn test_zome_get_function() {
        let mut zome = Zome::empty();
        zome.add_fn_declaration(String::from("test"), vec![], vec![]);
        let result = zome.get_function("foo func");
        assert!(result.is_none());
        let fun = zome.get_function("test").unwrap();
        assert_eq!(
            format!("{:?}", fun),
            "FnDeclaration { name: \"test\", inputs: [], outputs: [] }"
        );
    }
}
