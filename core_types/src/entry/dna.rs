#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dna {}

impl Dna {
    pub fn new() -> Dna {
        Dna{}
    }
}

pub fn test_dna() -> Dna {
    Dna::new()
}
