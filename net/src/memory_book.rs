
use holochain_core_types::{cas::content::Address, hash::HashString};

use holochain_net_connection::json_protocol::MetaTuple;

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    convert::TryFrom,
    sync::{mpsc, Mutex, RwLock},
};

pub(crate) type BucketId = String;

/// Type for holding list of addresses per entry per dna+agent_id
/// i.e. map of bucket_id -> (entry_address -> address / metaHash)
pub(crate) type AddressBook = HashMap<BucketId, HashMap<Address, Address>>;


/// return a BucketId out of a dna_address and agent_id
pub(crate) fn into_bucket_id(dna_address: &Address, agent_id: &str) -> BucketId {
    format!("{}::{}", dna_address, agent_id)
}

/// return a unique identifier out of an entry_address and attribute
pub(crate) fn into_meta_id(meta_tuple: &MetaTuple) -> Address {
    HashString::from(format!(
        "{}||{}||{}",
        meta_tuple.0, meta_tuple.1, meta_tuple.2
    ))
}

/// unmerge meta_id into a tuple
pub(crate) fn _undo_meta_id(meta_id: &Address) -> MetaTuple {
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
pub(crate) fn _metaId_is(metaId: &Address, entry_address: Address, attribute: String) -> bool {
    let meta_tuple = _undo_meta_id(metaId);
    return meta_tuple.0 == entry_address && meta_tuple.1 == attribute;
}

/// Add an address to a book
pub(crate) fn bookkeep_address_with_bucket(
    book: &mut AddressBook,
    bucket_id: BucketId,
    base_address: &Address,
    data_address: &Address
) {
    // Append to existing address list if there is one
    {
        let maybe_map = book.get_mut(&bucket_id);
        if let Some(map) = maybe_map {
            map.insert(base_address.clone(), data_address.clone());
            return;
        }
    } // unborrow book
    // None: Create and add a new address list
    let mut map = HashMap::new();
    map.insert(base_address.clone(), data_address.clone());
    book.insert(bucket_id, map);
}

// Return true if data is in book
pub fn book_has(
    &book: &AddressBook,
    bucket_id: BucketId,
    base_address: &Address,
    data_address: &Address,
) -> bool {
    let maybe_bucket_map = book.get(&bucket_id);
    if let None = maybe_bucket_map {
        return false;
    }
    let bucket_map = maybe_bucket_map.unwrap();
    let maybe_entry_map = bucket_map.get(base_address);
    if let None = maybe_entry_map {
        return false;
    }
    let entry_map = maybe_entry_map.unwrap();
    entry_map.contains(data_address)
}

///
pub fn book_has_entry(
    &book: &AddressBook,
    bucket_id: BucketId,
    entry_address: &Address,
) -> bool {
    book.has_bookkept(bucket_id, entry_address, entry_address)
}



/// Add an address to a book (sugar)
pub(crate) fn bookkeep_address(
    book: &mut AddressBook,
    dna_address: &Address,
    agent_id: &str,
    base_address: &Address,
    data_address: &Address,
) {
    let bucket_id = into_bucket_id(dna_address, agent_id);
    bookkeep_address_with_bucket(book, bucket_id, base_address, data_address);
}

/// Remove an address from a book
/// Return true if address exists and has been successfully removed.
pub(crate) fn _unbookkeep_address(
    book: &mut AddressBook,
    dna_address: &Address,
    agent_id: &str,
    base_address: &Address,
    data_address: &Address,
) -> bool {
    let bucket_id = into_bucket_id(dna_address, agent_id);
    let maybe_map = book.get_mut(&bucket_id);
    if let Some(bucket_map) = maybe_map {
        let maybe_entry_map = bucket_map.get_mut(base_address);
        if let Some(entry_map) = maybe_entry_map {
            let result = entry_map.remove_item(data_address);
            return result.is_some();
        }
    }
    false
}