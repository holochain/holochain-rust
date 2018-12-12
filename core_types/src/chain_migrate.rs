use agent::test_sources;
use crate::{
    agent::Sources,
    cas::content::{Address, AddressableContent},
    dna::zome::entry_types::Sharing,
    error::HolochainError,
    json::{JsonString, RawString},
};
use dna::test_dna;
use serde::{Deserialize, Deserializer, Serializer};

type Data = JsonString;

fn serialize_data<S>(data: &Data, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&String::from(data))
}

fn deserialize_data<'de, D>(deserializer: D) -> Result<Data, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct SerializedData(String);

    let serialized_data = SerializedData::deserialize(deserializer)?;
    Ok(Data::from(serialized_data.0))
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize, DefaultJson)]
pub enum MigrateType {
    Open,
    Close,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize, DefaultJson)]
pub struct ChainMigrate {
    sources: Sources,
    dna: Address,
    migrate_type: MigrateType,
    sharing: Sharing,

    #[serde(serialize_with = "serialize_data")]
    #[serde(deserialize_with = "deserialize_data")]
    data: Data,
}

impl ChainMigrate {
    pub fn new<J: Into<JsonString>>(
        sources: Sources,
        dna: Address,
        migrate_type: MigrateType,
        sharing: Sharing,
        data: J,
    ) -> ChainMigrate {
        ChainMigrate {
            sources,
            dna,
            migrate_type,
            sharing,
            data: data.into(),
        }
    }
}

pub fn test_migrate_type() -> MigrateType {
    MigrateType::Close
}

pub fn test_data() -> Data {
    Data::from(RawString::from("oops!"))
}

pub fn test_chain_migrate() -> ChainMigrate {
    ChainMigrate::new(
        test_sources(),
        test_dna().address(),
        test_migrate_type(),
        Sharing::default(),
        test_data(),
    )
}

#[cfg(test)]
mod test {

    use crate::{
        chain_migrate::{test_chain_migrate, ChainMigrate, MigrateType},
        json::JsonString,
    };
    use std::convert::TryFrom;
    use crate::chain_migrate::test_migrate_type;

    #[test]
    fn smoke_test() {
        test_migrate_type();
        test_chain_migrate();
    }

    #[test]
    fn json_round_trip_test() {
        for (variant, expected) in vec![
            (MigrateType::Open, "\"Open\""),
            (MigrateType::Close, "\"Close\""),
        ] {
            assert_eq!(JsonString::from(expected), JsonString::from(variant),);
        }

        let chain_migrate = test_chain_migrate();
        let expected = "{\"sources\":[\"MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd\"],\"dna\":\"Qmc6FxqxTni2hNx44MV47JgUrWTNvmyV9MR4B3Der7WBAJ\",\"migrate_type\":\"Close\",\"sharing\":\"public\",\"data\":\"\\\"oops!\\\"\"}";
        assert_eq!(JsonString::from(expected), JsonString::from(&chain_migrate),);

        assert_eq!(
            &chain_migrate,
            &ChainMigrate::try_from(JsonString::from(&chain_migrate)).unwrap(),
        )
    }
}
