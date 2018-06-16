use std;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceChain {
    pairs: Vec<super::Pair>
}

impl IntoIterator for SourceChain {
    type Item = super::Pair;
    type IntoIter = std::vec::IntoIter<super::Pair>;
    fn into_iter (self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

impl super::SourceChain for SourceChain {
}
