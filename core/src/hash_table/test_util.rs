use hash_table::HashTable;
use hash_table::pair::tests::test_pair;
use hash_table::pair_meta::PairMeta;
use hash_table::status::CRUDStatus;
use hash_table::pair::tests::test_pair_a;
use hash_table::pair::tests::test_pair_b;
use agent::keys::tests::test_keys;
use hash_table::status::LINK_NAME;
use hash_table::status::STATUS_NAME;
use key::Key;

// standard tests that should pass for every hash table implementation

pub fn test_round_trip<HT: HashTable> (table: &mut HT) {
    let pair = test_pair();
    table.commit_pair(&pair).expect("should be able to commit valid pair");
    assert_eq!(table.pair(&pair.key()), Ok(Some(pair)));
}

pub fn test_modify_pair<HT: HashTable> (table: &mut HT) {
    let pair_a = test_pair_a();
    let pair_b = test_pair_b();

    table.commit_pair(&pair_a).expect("should be able to commit valid pair");
    table.modify_pair(&test_keys(), &pair_a, &pair_b)
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
        table.all_metas_for_pair(&pair_a)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    let empty_vec: Vec<PairMeta> = Vec::new();
    assert_eq!(
        empty_vec,
        table.all_metas_for_pair(&pair_b)
            .expect("getting the metadata on a pair shouldn't fail")
    );
}

pub fn test_retract_pair<HT: HashTable> (table: &mut HT) {
    let pair = test_pair();
    let empty_vec: Vec<PairMeta> = Vec::new();

    table.commit_pair(&pair).expect("should be able to commit valid pair");
    assert_eq!(
        empty_vec,
        table.all_metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail")
    );

    table.retract_pair(&test_keys(), &pair)
        .expect("should be able to retract");
    assert_eq!(
        vec![PairMeta::new(
            &test_keys(),
            &pair,
            STATUS_NAME,
            &CRUDStatus::DELETED.bits().to_string(),
        )],
        table.all_metas_for_pair(&pair)
            .expect("getting the metadata on a pair shouldn't fail"),
    );
}
