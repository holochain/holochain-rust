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

impl IntoIterator for SourceChain {
    type Item = super::Pair;
    type IntoIter = std::vec::IntoIter<super::Pair>;
    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

impl super::SourceChain for SourceChain {
    fn push(&mut self, pair: &super::Pair) {
        self.pairs.insert(0, pair.clone())
    }
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

    fn test_pair(prev: Option<u64>, s: &str) -> Pair {
        let e = Entry::new(&s.to_string());
        let h = Header::new(prev, &e);
        Pair::new(&h, &e)
    }

    #[test]
    fn iter() {
        let mut chain = super::SourceChain::new();

        let p1 = test_pair(None, "foo");
        chain.push(&p1);

        let p2 = test_pair(Some(p1.header.hash()), "bar");
        chain.push(&p2);

        let p3 = test_pair(Some(p2.header.hash()), "foo");
        chain.push(&p3);

        assert_eq!(vec![&p3, &p2, &p1], chain.iter().collect::<Vec<&Pair>>());

        let foos = chain.iter().filter(|p| p.entry.content() == "foo").collect::<Vec<&Pair>>();

        assert_eq!(vec![&p3, &p1], foos);
    }

    #[test]
    fn into_iter() {
        let mut chain = super::SourceChain::new();

        let p1 = test_pair(None, "some content");
        chain.push(&p1);

        // into_iter() move
        let mut iter1 = chain.clone().into_iter();
        let i = iter1.next().unwrap();
        assert_eq!(p1, i);

        let p2 = test_pair(Some(p1.header.hash()), "some more content");
        chain.push(&p2);

        // into_iter() move
        let mut iter2 = chain.clone().into_iter();
        let i = iter2.next().unwrap();
        assert_eq!(p2, i);
        let i2 = iter2.next().unwrap();
        assert_eq!(p1, i2);

        // into_iter() move and filter
        let iter3 = chain.clone().into_iter();
        let f = iter3
            .filter(|p| p.entry.content() == "some content")
            .last()
            .unwrap();
        assert_eq!(f, p1)
    }
}
