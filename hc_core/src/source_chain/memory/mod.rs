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
    fn push (mut self, pair: super::Pair) {
        self.pairs.push(pair)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trip() {
        let _chain = super::SourceChain { pairs: Vec::new() };
    }
}
