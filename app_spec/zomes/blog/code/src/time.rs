//use boolinator::Boolinator;
use hdk::entry_definition::ValidatingEntryType;
/// This file holds everything that represents the "post" entry type.
use hdk::holochain_core_types::{
    dna::entry_types::Sharing, error::HolochainError, json::JsonString,
};

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone, PartialEq)]
pub enum TimeType {
    Year, 
    Month,
    Day,
    Hour
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Time {
    pub time: String,
    pub time_type: TimeType,
}

impl Time {
    pub fn new(time: &str, time_type: &TimeType) -> Time {
        Time {
            time: time.to_owned(),
            time_type: time_type.to_owned(),
        }
    }

    pub fn content(&self) -> String {
        self.time.clone()
    }

    pub fn time_type(&self) -> TimeType{
        self.time_type.clone()
    }
}

pub fn definition() -> ValidatingEntryType {
    entry!(
        name: "time",
        description: "A time entry - used for time based indexing",
        sharing: Sharing::Public,

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |_validation_data: hdk::EntryValidationData<Time>| {
            Ok(())
        },

        links: [
            to!(
                "post",
                tag: "*", //Any tag or expression tag
                r#type: "time_index",
                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },
                validation: |_validation_data: hdk::LinkValidationData| {
                    Ok(())
                }   
            )
        ]
    )
}

#[cfg(test)]
mod tests {
    use crate::time::{Time, TimeType};

    #[test]
    fn time_smoke_test() {
        let content = "01";
        let time_type = TimeType::Month;
        let time = Time::new(content, &time_type);

        assert_eq!(content.to_string(), time.content(),);

        assert_eq!(time_type, time.time_type(),);
    }

    #[test]
    fn time_definition_test() {

    }
}