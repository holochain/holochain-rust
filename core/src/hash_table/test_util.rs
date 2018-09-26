use agent::keys::tests::test_keys;
use hash_table::{
    entry::tests::test_entry_unique,
    entry_meta::{
        tests::{
            test_attribute, test_attribute_b, test_meta, test_meta_for, test_value, test_value_b,
        },
        EntryMeta,
    },
    status::{CrudStatus, LINK_NAME, STATUS_NAME},
    HashTable,
};
use key::Key;

// standard tests that should pass for every hash table implementation

pub fn test_round_trip<HT: HashTable>(table: &mut HT) {
    let entry = test_entry_unique();
    table
        .put_entry(&entry)
        .expect("should be able to commit valid entry");
    assert_eq!(table.entry(&entry.key()), Ok(Some(entry)));
}

pub fn test_modify<HT: HashTable>(table: &mut HT) {
    let entry_1 = test_entry_unique();
    let entry_2 = test_entry_unique();

    table.put_entry(&entry_1).unwrap();
    table
        .modify_entry(&test_keys(), &entry_1, &entry_2)
        .unwrap();

    assert_eq!(
        vec![
            EntryMeta::new(
                &test_keys().node_id(),
                &entry_1.key(),
                LINK_NAME,
                &entry_2.key().to_str(),
            ),
            EntryMeta::new(
                &test_keys().node_id(),
                &entry_1.key(),
                STATUS_NAME,
                &CrudStatus::MODIFIED.bits().to_string(),
            ),
        ],
        table.metas_from_entry(&entry_1).unwrap()
    );

    let empty_vec: Vec<EntryMeta> = Vec::new();
    assert_eq!(empty_vec, table.metas_from_entry(&entry_2).unwrap());
}

pub fn test_retract<HT: HashTable>(table: &mut HT) {
    let entry = test_entry_unique();
    let empty_vec: Vec<EntryMeta> = Vec::new();

    table.put_entry(&entry).unwrap();
    assert_eq!(empty_vec, table.metas_from_entry(&entry).unwrap());

    table
        .retract_entry(&test_keys(), &entry)
        .expect("should be able to retract");
    assert_eq!(
        vec![EntryMeta::new(
            &test_keys().node_id(),
            &entry.key(),
            STATUS_NAME,
            &CrudStatus::DELETED.bits().to_string(),
        )],
        table.metas_from_entry(&entry).unwrap(),
    );
}

pub fn test_meta_round_trip<HT: HashTable>(table: &mut HT) {
    let meta = test_meta();

    assert_eq!(None, table.get_meta(&meta.key()).unwrap());

    table
        .assert_meta(&meta)
        .expect("asserting metadata shouldn't fail");
    assert_eq!(Some(&meta), table.get_meta(&meta.key()).unwrap().as_ref());
}

/// assert a couple of unique metas against a single entry
fn test_metas_for<HT: HashTable>(table: &mut HT) {
    let entry = test_entry_unique();
    let meta_a = test_meta_for(&entry, &test_attribute(), &test_value());
    let meta_b = test_meta_for(&entry, &test_attribute_b(), &test_value_b());
    let empty_vec: Vec<EntryMeta> = Vec::new();

    assert_eq!(
        empty_vec,
        table
            .metas_from_entry(&entry)
            .expect("getting the metadata on a entry shouldn't fail")
    );

    table
        .assert_meta(&meta_a)
        .expect("asserting metadata shouldn't fail");
    assert_eq!(
        vec![meta_a.clone()],
        table
            .metas_from_entry(&entry)
            .expect("getting the metadata on a entry shouldn't fail")
    );

    table
        .assert_meta(&meta_b)
        .expect("asserting metadata shouldn't fail");
    assert_eq!(
        vec![meta_b, meta_a],
        table
            .metas_from_entry(&entry)
            .expect("getting the metadata on a entry shouldn't fail")
    );
}

pub fn standard_suite<HT: HashTable>(table: &mut HT) {
    assert_eq!(Ok(()), table.setup());

    test_round_trip(table);

    test_modify(table);

    test_retract(table);

    test_meta_round_trip(table);

    test_metas_for(table);

    assert_eq!(Ok(()), table.teardown());
}
