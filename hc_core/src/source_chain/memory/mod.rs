use std;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceChain {
    pairs: Vec<super::Pair>
}

impl SourceChain {
    pub fn new(pairs: &Vec<super::Pair>) -> SourceChain {
        SourceChain {
            pairs: pairs.clone(),
        }
    }
}

impl IntoIterator for SourceChain {
    type Item = super::Pair;
    type IntoIter = std::vec::IntoIter<super::Pair>;
    fn into_iter (self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

impl super::SourceChain for SourceChain {
    fn push (&mut self, pair: &super::Pair) {
        self.pairs.push(pair.clone())
    }
}

#[cfg(test)]
mod tests {
    use common::entry::Entry;
    use common::entry::Header;
    use source_chain::Pair;
    use source_chain::SourceChain;

    #[test]
    fn round_trip() {
        let mut chain = super::SourceChain::new(&Vec::new());

        let e1 = Entry::new(&String::from("some content"));
        println!("{:?}", e1);
        let h1 = Header::new(None, &e1);
        println!("{:?}", h1);
        let p1 = Pair::new(&h1, &e1);
        chain.push(&p1);
        println!("{:?}", chain);

        let e2 = Entry::new(&String::from("some more content"));
        let h2 = Header::new(Some(h1.hash()), &e2);
        let p2 = Pair::new(&h2, &e2);
        chain.push(&p2);
        println!("{:?}", chain);
        //
        // for pair in chain {
        //     println!("{:?}", pair)
        // }
    }
}
