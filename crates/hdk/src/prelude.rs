//! Types required by all but the most trivial zomes.
//! This can greatly simplify imports for the majority of developers
//! by simply adding use hdk::prelude::*;

// macros
pub use crate::{define_zome, entry, from, link, to};

// derive macros
pub use holochain_json_derive::DefaultJson;
pub use serde_derive::{Deserialize, Serialize};
pub use std::convert::{TryInto, TryFrom};
pub use holochain_wasmer_guest::*;

// types
pub use crate::{
    DNA_ADDRESS,
    AGENT_ADDRESS,
    DNA_NAME,
    AGENT_ID_STR,
    CAPABILITY_REQ,
    PROPERTIES,
    entry_definition::ValidatingEntryType,
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        agent::AgentId,
        dna::entry_types::Sharing,
        dna::capabilities::CapabilityRequest,
        entry::{entry_type::EntryType, entry_type::AppEntryType, Entry, AppEntryValue},
        error::HolochainError,
        link::LinkMatch,
        network::query::{Pagination, SizePagination, SortOrder, TimePagination},
        time::Iso8601,
        validation::ValidationResult,
    },
    holochain_json_api::{error::JsonError, json::JsonString, json::RawString},
    holochain_persistence_api::cas::content::{Address, AddressableContent},
    holochain_wasm_types::{
        commit_entry::CommitEntryOptions,
        get_entry::{
            EntryHistory, GetEntryOptions, GetEntryResult, GetEntryResultType, StatusRequestKind,
        },
        get_links::{GetLinksOptions, GetLinksResult, GetLinksResultCount, LinksStatusRequestKind},
        QueryArgsOptions, QueryResult, QueryArgsNames,
    },
    EntryValidationData, LinkValidationData, ValidationPackageDefinition,
};
