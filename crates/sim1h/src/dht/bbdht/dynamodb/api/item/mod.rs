use rusoto_dynamodb::AttributeValue;
use std::collections::HashMap;

pub mod fixture;
pub mod read;
pub mod write;

pub type Item = HashMap<String, AttributeValue>;
