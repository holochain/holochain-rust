use std;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceChain {
    pairs: Vec<super::Pair>,
}

impl SourceChain {
    pub fn new() -> SourceChain {
        SourceChain {
            pairs: Vec::new(),
        }
    }
}

// for loop support that consumes chains
impl IntoIterator for SourceChain {
    type Item = super::Pair;
    type IntoIter = std::vec::IntoIter<super::Pair>;
    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

// iter() style support for references to chains
impl<'a> IntoIterator for &'a SourceChain {
    type Item = &'a super::Pair;
    type IntoIter = std::slice::Iter<'a, super::Pair>;

    fn into_iter(self) -> std::slice::Iter<'a, super::Pair> {
        self.pairs.iter()
    }
}

// basic SouceChain trait
impl super::SourceChain for SourceChain {
    // appends the current pair to the top of the chain
    // @TODO - appending pairs should fail if hashes do not line up
    // @see https://github.com/holochain/holochain-rust/issues/31
    fn push(&mut self, pair: &super::Pair) {
        self.pairs.append(&mut vec![pair.clone()])
    }
    // returns an iterator referencing pairs from bottom (genesis) to top (most recent)
    fn iter(&self) -> std::slice::Iter<super::Pair> {
        self.pairs.iter()
    }
}

#[cfg(test)]
mod tests {
    use common::entry::Entry;
    use common::entry::Header;
    use source_chain::Pair;
    use source_chain::SourceChain;

    // helper to spin up pairs for testing
    // @TODO - do we want to expose something like this as a general utility?
    // @see https://github.com/holochain/holochain-rust/issues/34
    fn test_pair(previous_pair: Option<&Pair>, s: &str) -> Pair {
        let e = Entry::new(&s.to_string());
        let previous = match previous_pair {
            Some(p) => Some(p.header.hash()),
            None => None,
        };
        let h = Header::new(previous, &e);
        Pair::new(&h, &e)
    }

    #[test]
    fn iter() {
        // setup
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");
        let p3 = test_pair(Some(&p2), "foo");

        let mut chain = super::SourceChain::new();
        chain.push(&p1);
        chain.push(&p2);
        chain.push(&p3);

        // iter() should iterate over references
        assert_eq!(vec![&p3, &p2, &p1], chain.iter().rev().collect::<Vec<&Pair>>());

        // iter() should support functional logic
        assert_eq!(
            vec![&p1, &p3],
            chain
                .iter()
                .filter(|p| p.entry.content() == "foo")
                .collect::<Vec<&Pair>>()
        );
    }

    #[test]
    fn into_iter() {
        // setup
        let p1 = test_pair(None, "foo");
        let p2 = test_pair(Some(&p1), "bar");
        let p3 = test_pair(Some(&p2), "baz");

        let mut chain = super::SourceChain::new();
        chain.push(&p1);
        chain.push(&p2);
        chain.push(&p3);

        // into_iter() by reference
        let mut i = 0;
        let expected = [&p1, &p2, &p3];
        for p in &chain {
            assert_eq!(expected[i], p);
            i = i + 1;
        }

        // do functional things with (&chain).into_iter()
        assert_eq!(
            vec![&p1],
            (&chain)
                .into_iter().rev()
                .filter(|p| p.header.previous() == None)
                .collect::<Vec<&Pair>>()
        );

        // into_iter() move
        let mut i = 0;
        let expected = [p1.clone(), p2.clone(), p3.clone()];
        for p in chain.clone() {
            assert_eq!(expected[i], p);
            i = i + 1;
        }
    }
}
