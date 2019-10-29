use std::{collections::HashSet, hash::Hash};

pub fn unordered_vec_compare<T: Hash + Eq>(a: Vec<T>, b: Vec<T>) -> bool {
    let mut set_a = HashSet::new();
    for i in a {
        set_a.insert(i);
    }
    let mut set_b = HashSet::new();
    for j in b {
        set_b.insert(j);
    }
    set_a == set_b
}
