use agent::keys::tests::test_keys;
use hash_table::{
    pair::tests::{test_pair_unique},
    pair_meta::{
        tests::{test_pair_meta, test_pair_meta_for, test_attribute, test_value, test_attribute_b, test_value_b},
        PairMeta,
    },
    status::{CRUDStatus, LINK_NAME, STATUS_NAME},
    HashTable,
};
use key::Key;

// standard tests that should pass for every hash table implementation

pub fn test_pair_round_trip<HT: HashTable>(table: &mut HT) {
    let pair = test_pair_unique();
    table
        .commit_pair(&pair)
        .expect("should be able to commit valid pair");
    assert_eq!(table.pair(&pair.key()), Ok(Some(pair)));
}

pub fn test_modify_pair<HT: HashTable>(table: &mut HT) {
    let pair_1 = test_pair_unique();
    let pair_2 = test_pair_unique();

    table
        .commit_pair(&pair_1)
        .expect("should be able to commit valid pair");
    table
        .modify_pair(&test_keys(), &pair_1, &pair_2)
        .expect("should be able to edit with valid pair");

    assert_eq!(
        vec![
            PairMeta::new(&test_keys(), &pair_1, LINK_NAME, &pair_2.key()),
            PairMeta::new(
                &test_keys(),
                &pair_1,
                STATUS_NAME,
                &CRUDStatus::MODIFIED.bits().to_string(),
            ),
        ],
        table
            .metas_for_pair(&pair_1)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    let empty_vec: Vec<PairMeta> = Vec::new();
    assert_eq!(
        empty_vec,
        table
            .metas_for_pair(&pair_2)
            .expect("getting the metadata on a pair shouldn't fail")
    );
}

pub fn test_retract_pair<HT: HashTable>(table: &mut HT) {
    let pair = test_pair_unique();
    let empty_vec: Vec<PairMeta> = Vec::new();

    table
        .commit_pair(&pair)
        .expect("should be able to commit valid pair");
    assert_eq!(
        empty_vec,
        table
            .metas_for_pair(&pair)
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
            .metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail"),
    );
}

pub fn test_pair_meta_round_trip<HT: HashTable>(table: &mut HT) {
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

pub fn test_metas_for_pair<HT: HashTable>(table: &mut HT) {
    let pair = test_pair_unique();
    let meta_a = test_pair_meta_for(&pair, &test_attribute(), &test_value());
    let meta_b = test_pair_meta_for(&pair, &test_attribute_b(), &test_value_b());
    let empty_vec: Vec<PairMeta> = Vec::new();

    assert_eq!(
        empty_vec,
        table
            .metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    table
        .assert_pair_meta(&meta_a)
        .expect("asserting metadata shouldn't fail");
    assert_eq!(
        vec![meta_a.clone()],
        table
            .metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    table
        .assert_pair_meta(&meta_b)
        .expect("asserting metadata shouldn't fail");
    assert_eq!(
        vec![meta_b, meta_a],
        table
            .metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail")
    );
}

pub fn standard_suite<HT: HashTable>(table: &mut HT) {

    assert_eq!(
        Ok(()),
        table.setup()
    );

    test_pair_round_trip(table);

    test_modify_pair(table);

    test_retract_pair(table);

    test_pair_meta_round_trip(table);

    test_metas_for_pair(table);

    assert_eq!(
        Ok(()),
        table.teardown()
    );

}
