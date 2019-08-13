use error::{HcResult,HolochainError};

pub enum Lifecycle
{
    Beta(i8),
    Alpha(i8),
    Stable(i8)
}

pub struct HDKVersion
{
    versioning : (i8,i8,i8),
    lifecycle : Lifecycle
}

fn get_lifecycle(lifecycle_string:&str) ->HcResult<Lifecycle>
{
    if lifecycle_string.contains("beta")
    {
        Ok(Lifecycle::Beta(lifecycle_string.split("beta").nth(1).ok_or("Could not get beta version")?
                       .parse::<i8>().map_err(|_|HolochainError::ErrorGeneric("Could not parse version".to_string()))?))
    }
    else if lifecycle_string.contains("stable")
    {
        Ok(Lifecycle::Stable(lifecycle_string.split("stable").nth(1).ok_or("Could not get stable version")?
                      .parse::<i8>().map_err(|_|HolochainError::ErrorGeneric("Could not parse version".to_string()))?))
    }
    else if lifecycle_string.contains("alpha")
    {
        Ok(Lifecycle::Alpha(lifecycle_string.split("alpha").nth(1).ok_or("Could not get alpha version")?
                     .parse::<i8>().map_err(|_|HolochainError::ErrorGeneric("Could not parse version".to_string()))?))
    }
    else 
    {
        Err(HolochainError::ErrorGeneric("Invalid Lifecycle Version".to_string()))
    }
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

        let lifecycle = get_lifecycle(splits.next().ok_or("Could not get lifecycle")?)?;

        Ok(HDKVersion
        {
            versioning,
            lifecycle,
        })
    }
}