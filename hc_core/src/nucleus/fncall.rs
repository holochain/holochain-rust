/// Represents a function call, with the function name and its parameters

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    name: String,
    params: Params,
}

impl Call {
    pub fn new(name: &str) -> Self {
        Call{name:name.to_string(), params: Params{}}
    }
}
