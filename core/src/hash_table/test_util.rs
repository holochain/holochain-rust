use agent::keys::tests::test_keys;
use hash_table::{
    pair::tests::{test_pair, test_pair_a, test_pair_b},
    pair_meta::{
        tests::{test_pair_meta, test_pair_meta_a, test_pair_meta_b},
        PairMeta,
    },
    status::{CRUDStatus, LINK_NAME, STATUS_NAME},
    HashTable,
};
use key::Key;

// standard tests that should pass for every hash table implementation

pub fn test_pair_round_trip<HT: HashTable>(table: &mut HT) {
    let pair = test_pair();
    table
        .commit_pair(&pair)
        .expect("should be able to commit valid pair");
    assert_eq!(table.pair(&pair.key()), Ok(Some(pair)));
}

pub fn test_modify_pair<HT: HashTable>(table: &mut HT) {
    let pair_a = test_pair_a();
    let pair_b = test_pair_b();

    table
        .commit_pair(&pair_a)
        .expect("should be able to commit valid pair");
    table
        .modify_pair(&test_keys(), &pair_a, &pair_b)
        .expect("should be able to edit with valid pair");

    assert_eq!(
        vec![
            PairMeta::new(&test_keys(), &pair_a, LINK_NAME, &pair_b.key()),
            PairMeta::new(
                &test_keys(),
                &pair_a,
                STATUS_NAME,
                &CRUDStatus::MODIFIED.bits().to_string(),
            ),
        ],
        table
            .all_metas_for_pair(&pair_a)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    let empty_vec: Vec<PairMeta> = Vec::new();
    assert_eq!(
        empty_vec,
        table
            .all_metas_for_pair(&pair_b)
            .expect("getting the metadata on a pair shouldn't fail")
    );
}

pub fn test_retract_pair<HT: HashTable>(table: &mut HT) {
    let pair = test_pair();
    let empty_vec: Vec<PairMeta> = Vec::new();

    table
        .commit_pair(&pair)
        .expect("should be able to commit valid pair");
    assert_eq!(
        empty_vec,
        table
            .all_metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    table
        .retract_pair(&test_keys(), &pair)
        .expect("should be able to retract");
    assert_eq!(
        vec![PairMeta::new(
            &test_keys(),
            &pair,
            STATUS_NAME,
            &CRUDStatus::DELETED.bits().to_string(),
        )],
        table
            .all_metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail"),
    );
}

pub fn test_meta_round_trip<HT: HashTable>(table: &mut HT) {
    let meta = test_pair_meta();

    assert_eq!(
        None,
        table
            .pair_meta(&meta.key())
            .expect("getting the metadata on a pair shouldn't fail")
    );

    table
        .assert_pair_meta(&meta)
        .expect("asserting metadata shouldn't fail");
    assert_eq!(
        Some(&meta),
        table
            .pair_meta(&meta.key())
            .expect("getting the metadata on a pair shouldn't fail")
            .as_ref()
    );
}

pub fn test_all_metas_for_pair<HT: HashTable>(table: &mut HT) {
    let pair = test_pair();
    let meta_a = test_pair_meta_a();
    let meta_b = test_pair_meta_b();
    let empty_vec: Vec<PairMeta> = Vec::new();

    assert_eq!(
        empty_vec,
        table
            .all_metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    table
        .assert_pair_meta(&meta_a)
        .expect("asserting metadata shouldn't fail");
    assert_eq!(
        vec![meta_a.clone()],
        table
            .all_metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    table
        .assert_pair_meta(&meta_b)
        .expect("asserting metadata shouldn't fail");
    assert_eq!(
        vec![meta_b, meta_a],
        table
            .all_metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail")
    );
}
