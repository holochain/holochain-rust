use error::{HcResult,HolochainError};

pub enum Lifecycle
{
    Beta,
    Alpha,
    Stable
}

pub struct HDKVersion
{
    versioning : (i8,i8,i8),
    life_cycle : Lifecycle
}

impl HDKVersion
{
    pub fn new(version_string:&str) ->HcResult<HDKVersion>
    {
        let mut splits = version_string.split("-");
        let version = splits.next().ok_or("Could not get version")?;

        let mut version_splits = version.split(".");
        let versioning = (version_splits.next().ok_or("Could not get version")?.parse::<i8>().map_err(|_|HolochainError::ErrorGeneric("Could not parse version".to_string()))?,
                          version_splits.next().ok_or("Could not get version")?.parse::<i8>().map_err(|_|HolochainError::ErrorGeneric("Could not parse version".to_string()))?,
                          version_splits.next().ok_or("Could not get version")?.parse::<i8>().map_err(|_|HolochainError::ErrorGeneric("Could not parse version".to_string()))?);

        let life_cycle = match splits.next().ok_or("Could not get lifecycle")?
        {
            "Beta" => Ok(Lifecycle::Beta),
            "Stable" => Ok(Lifecycle::Stable),
            "Alpha" => Ok(Lifecycle::Alpha),
            _ => Err(HolochainError::ErrorGeneric("invalid lifecycle".to_string()))
        }?;

        Ok(HDKVersion
        {
            versioning,
            life_cycle,
        })
    }
}