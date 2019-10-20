use holochain_persistence_api::{cas::content::Address, hash::HashString};
use lib3h_protocol::types::AgentPubKey;
use lib3h_protocol::types::EntryHash;
use lib3h_protocol::types::HashStringNewType;
use lib3h_protocol::types::AspectHash;

use std::collections::{HashMap, HashSet};

//--------------------------------------------------------------------------------------------------
// IDs
//--------------------------------------------------------------------------------------------------

/// Cell = DnaInstance
/// CellId = DNA hash + agentId
pub(crate) type ChainId = String;

/// return a cell_id out of a dna_address and agent_id
pub(crate) fn into_chain_id(dna_address: &Address, agent_id: &Address) -> ChainId {
    format!("{}::{}", dna_address, agent_id)
}

/// unmerge meta_id into a tuple
pub(crate) fn undo_chain_id(chain_id: &ChainId) -> (Address, AgentPubKey) {
    let chain_str = chain_id.clone();
    let vec: Vec<&str> = chain_str.split("::").collect();
    assert_eq!(vec.len(), 2);
    // Convert & return
    (HashString::from(vec[0]), AgentPubKey::from(vec[1]))
}

//--------------------------------------------------------------------------------------------------
// Books
//--------------------------------------------------------------------------------------------------

/// Type for holding list of EntryAspects per Entry.
/// i.e. map of entry_address -> Set(aspect_address)
/// entry_address -> Set(aspect_address) -- edge case for indicating we are storing an entry?
pub(crate) type EntryBook = HashMap<EntryHash, HashSet<AspectHash>>;

/// Type for holding list of addresses per entry per ChainId (dna+agent_id)
/// i.e. map of ChainId -> EntryBook
pub(crate) type ChainBook = HashMap<ChainId, EntryBook>;

/// Add an address to a book
pub(crate) fn bookkeep_with_chain_id(
    chain_book: &mut ChainBook,
    chain_id: ChainId,
    entry_address: &EntryHash,
    aspect_address: &AspectHash,
) {
    // Append to existing address list if there is one
    {
        let maybe_entry_book = chain_book.get_mut(&chain_id);
        if let Some(entry_book) = maybe_entry_book {
            // Append to existing address list if there is one
            {
                let maybe_aspect_set = entry_book.get_mut(entry_address);
                if let Some(meta_set) = maybe_aspect_set {
                    meta_set.insert(aspect_address.clone());
                    return;
                }
            }
            let mut aspect_set = HashSet::new();
            aspect_set.insert(aspect_address.clone());
            entry_book.insert(entry_address.clone(), aspect_set.clone());
            return;
        }
    } // unborrow book
      // None: Create and add a new EntryBook
    let mut entry_book = EntryBook::new();
    let mut aspect_set = HashSet::new();
    aspect_set.insert(aspect_address.clone());
    entry_book.insert(entry_address.clone(), aspect_set);
    chain_book.insert(chain_id, entry_book);
}

/// Add an address to a book (sugar)
pub(crate) fn bookkeep(
    chain_book: &mut ChainBook,
    dna_address: &Address,
    agent_id: &AgentPubKey,
    entry_address: &EntryHash,
    aspect_address: &AspectHash,
) {
    let chain_id = into_chain_id(dna_address, agent_id);
    bookkeep_with_chain_id(chain_book, chain_id, entry_address, aspect_address);
}

// Return true if data is in book
pub fn book_has_aspect(
    chain_book: &ChainBook,
    chain_id: ChainId,
    entry_address: &EntryHash,
    aspect_address: &AspectHash,
) -> bool {
    let maybe_entry_book = chain_book.get(&chain_id);
    if maybe_entry_book.is_none() {
        return false;
    }
    let entry_book = maybe_entry_book.unwrap();
    let maybe_aspect_set = entry_book.get(entry_address);
    if maybe_aspect_set.is_none() {
        return false;
    }
    let aspect_set = maybe_aspect_set.unwrap();
    aspect_set.contains(aspect_address)
}

///
pub fn book_has_entry(chain_book: &ChainBook, chain_id: ChainId, entry_address: &EntryHash) -> bool {
    book_has_aspect(&chain_book, chain_id, entry_address, &AspectHash::from(entry_address.hash_string()))
}

/// Remove an address from a book
/// Return true if address exists and has been successfully removed.
pub(crate) fn _unbookkeep_address(
    chain_book: &mut ChainBook,
    dna_address: &Address,
    agent_id: &AgentPubKey,
    entry_address: &EntryHash,
    aspect_address: &AspectHash,
) -> bool {
    let chain_id = into_chain_id(dna_address, agent_id);
    let maybe_entry_book = chain_book.get_mut(&chain_id);
    if let Some(entry_book) = maybe_entry_book {
        let maybe_aspect_set = entry_book.get_mut(entry_address);
        if let Some(aspect_set) = maybe_aspect_set {
            let succeeded = aspect_set.remove(aspect_address);
            return succeeded;
        }
    }
    false
}
