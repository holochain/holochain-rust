use holochain_core_types::{cas::content::Address, hash::HashString};

use crate::connection::json_protocol::MetaTuple;

use std::collections::{HashMap, HashSet};

//--------------------------------------------------------------------------------------------------
// Ids
//--------------------------------------------------------------------------------------------------

/// Cell = DnaInstance
/// CellId = DNA hash + agentId
pub(crate) type CellId = String;

/// Hash of a meta content in a way to uniquely identify it
pub(crate) type MetaId = Address;

/// return a cell_id out of a dna_address and agent_id
pub(crate) fn into_cell_id(dna_address: &Address, agent_id: &str) -> CellId {
    format!("{}::{}", dna_address, agent_id)
}

/// return a unique identifier out of an entry_address and attribute
pub(crate) fn into_meta_id(meta_tuple: &MetaTuple) -> MetaId {
    HashString::from(format!(
        "{}||{}||{}",
        meta_tuple.0, meta_tuple.1, meta_tuple.2
    ))
}

/// unmerge meta_id into a tuple
pub(crate) fn _undo_meta_id(meta_id: &MetaId) -> MetaTuple {
    let meta_str = String::from(meta_id.clone());
    let vec: Vec<&str> = meta_str.split("||").collect();
    assert_eq!(vec.len(), 3);
    // Convert & return
    (
        HashString::from(vec[0]),
        vec[1].to_string(),
        serde_json::from_str(vec[2]).expect("metaId not holding valid json content"),
    )
}

/// Tells if metaId is of given entry and attribute
pub(crate) fn _meta_id_is(meta_id: &MetaId, entry_address: Address, attribute: String) -> bool {
    let meta_tuple = _undo_meta_id(meta_id);
    return meta_tuple.0 == entry_address && meta_tuple.1 == attribute;
}

//--------------------------------------------------------------------------------------------------
// Books
//--------------------------------------------------------------------------------------------------

/// Type for holding list of metaId per entry
/// i.e. map of entry_address -> Set(metaId)
/// edge case is entry_address -> entry_address, which is the way to signal that we are holding
/// the entry itself
pub(crate) type EntryBook = HashMap<Address, HashSet<HashString>>;

/// Type for holding list of addresses per entry per dna+agent_id
/// i.e. map of cell_id -> EntryBook
pub(crate) type CellBook = HashMap<CellId, EntryBook>;

/// Add an address to a book
pub(crate) fn bookkeep_with_cell_id(
    cell_book: &mut CellBook,
    cell_id: CellId,
    base_address: &Address,
    data_address: &Address,
) {
    // Append to existing address list if there is one
    {
        let maybe_entry_book = cell_book.get_mut(&cell_id);
        if let Some(entry_book) = maybe_entry_book {
            // Append to existing address list if there is one
            {
                let maybe_meta_set = entry_book.get_mut(&base_address);
                if let Some(meta_set) = maybe_meta_set {
                    meta_set.insert(data_address.clone());
                    return;
                }
            }
            let mut meta_set = HashSet::new();
            meta_set.insert(data_address.clone());
            entry_book.insert(base_address.clone(), meta_set.clone());
            return;
        }
    } // unborrow book
      // None: Create and add a new EntryBook
    let mut entry_book = EntryBook::new();
    let mut meta_set = HashSet::new();
    meta_set.insert(data_address.clone());
    entry_book.insert(base_address.clone(), meta_set);
    cell_book.insert(cell_id, entry_book);
}

/// Add an address to a book (sugar)
pub(crate) fn bookkeep(
    cell_book: &mut CellBook,
    dna_address: &Address,
    agent_id: &str,
    base_address: &Address,
    data_address: &Address,
) {
    let cell_id = into_cell_id(dna_address, agent_id);
    bookkeep_with_cell_id(cell_book, cell_id, base_address, data_address);
}

// Return true if data is in book
pub fn book_has(
    cell_book: &CellBook,
    cell_id: CellId,
    base_address: &Address,
    data_address: &Address,
) -> bool {
    let maybe_entry_book = cell_book.get(&cell_id);
    if maybe_entry_book.is_none() {
        return false;
    }
    let entry_book = maybe_entry_book.unwrap();
    let maybe_meta_set = entry_book.get(base_address);
    if maybe_meta_set.is_none() {
        return false;
    }
    let meta_set = maybe_meta_set.unwrap();
    meta_set.contains(data_address)
}

///
pub fn book_has_entry(cell_book: &CellBook, cell_id: CellId, entry_address: &Address) -> bool {
    book_has(&cell_book, cell_id, entry_address, entry_address)
}

/// Remove an address from a book
/// Return true if address exists and has been successfully removed.
pub(crate) fn _unbookkeep_address(
    cell_book: &mut CellBook,
    dna_address: &Address,
    agent_id: &str,
    base_address: &Address,
    data_address: &Address,
) -> bool {
    let cell_id = into_cell_id(dna_address, agent_id);
    let maybe_entry_book = cell_book.get_mut(&cell_id);
    if let Some(entry_book) = maybe_entry_book {
        let maybe_meta_set = entry_book.get_mut(base_address);
        if let Some(meta_set) = maybe_meta_set {
            let succeeded = meta_set.remove(data_address);
            return succeeded;
        }
    }
    false
}
