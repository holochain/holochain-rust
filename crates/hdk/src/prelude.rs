//! Types required by all but the most trivial zomes.
//! This can greatly simplify imports for the majority of developers
//! by simply adding use hdk::prelude::*;

// macros
pub use crate::{define_zome, entry, from, link, load_json, to};

// derive macros
pub use holochain_json_derive::DefaultJson;
pub use serde_derive::{Deserialize, Serialize};

// types
pub use crate::{
    entry_definition::ValidatingEntryType,
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        agent::AgentId,
        dna::entry_types::Sharing,
        entry::{entry_type::EntryType, Entry},
        error::HolochainError,
        link::LinkMatch,
    },
    holochain_json_api::{error::JsonError, json::JsonString},
    holochain_persistence_api::cas::content::{Address, AddressableContent},
    holochain_wasm_utils::api_serialization::{
        commit_entry::CommitEntryOptions,
        get_entry::{
            EntryHistory, GetEntryOptions, GetEntryResult, GetEntryResultType, StatusRequestKind,
        },
        get_links::{GetLinksOptions, GetLinksResult, GetLinksResultCount, LinksStatusRequestKind},
        QueryArgsOptions, QueryResult,
    },
    EntryValidationData, LinkValidationData, ValidationPackageDefinition,
};
