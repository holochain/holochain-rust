use hash_table::HashTable;
use hash_table::pair::tests::test_pair;

// standard tests that should pass for every hash table implementation

pub fn test_round_trip<HT: HashTable> (table: &mut HT) {
    let pair = test_pair();
    table.commit_pair(&pair).expect("should be able to commit valid pair");
    assert_eq!(table.pair(&pair.key()), Ok(Some(pair)));
}
