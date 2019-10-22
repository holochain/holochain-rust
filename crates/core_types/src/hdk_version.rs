use error::{HcResult, HolochainError};

lazy_static! {
    pub static ref HDK_VERSION: HDKVersion = {
        let version = env!(
            "HDK_VERSION",
            "failed to obtain hdk version from build environment. Check build.rs"
        );
        HDKVersion::new(version).unwrap_or_else(|_| {
            panic!("Failed to create HDK Version '{}'. Check Build.rs", version)
        })
    };
}

#[derive(Clone, PartialEq, Eq)]
pub enum Lifecycle {
    Beta(i16),
    Alpha(i16),
    Stable(i16),
}

#[derive(Clone, PartialEq, Eq)]
pub struct HDKVersion {
    versioning: (i16, i16, i16),
    lifecycle: Lifecycle,
}

fn get_lifecycle(lifecycle_string: &str) -> HcResult<Lifecycle> {
    if lifecycle_string.contains("beta") {
        Ok(Lifecycle::Beta(
            lifecycle_string
                .split("beta")
                .nth(1)
                .ok_or("Could not get beta version")?
                .parse::<i16>()
                .map_err(|_| HolochainError::ErrorGeneric("Could not parse version".to_string()))?,
        ))
    } else if lifecycle_string.contains("stable") {
        Ok(Lifecycle::Stable(
            lifecycle_string
                .split("stable")
                .nth(1)
                .ok_or("Could not get stable version")?
                .parse::<i16>()
                .map_err(|_| HolochainError::ErrorGeneric("Could not parse version".to_string()))?,
        ))
    } else if lifecycle_string.contains("alpha") {
        Ok(Lifecycle::Alpha(
            lifecycle_string
                .split("alpha")
                .nth(1)
                .ok_or("Could not get alpha version")?
                .parse::<i16>()
                .map_err(|_| HolochainError::ErrorGeneric("Could not parse version".to_string()))?,
        ))
    } else {
        Err(HolochainError::ErrorGeneric(
            "Invalid Lifecycle Version".to_string(),
        ))
    }
}

impl ToString for HDKVersion {
    fn to_string(&self) -> String {
        let version = vec![
            self.versioning.0.to_string(),
            self.versioning.1.to_string(),
            self.versioning.2.to_string(),
        ]
        .join(".");
        let life_cycle = match self.lifecycle {
            Lifecycle::Alpha(num) => vec!["alpha", &num.to_string()].join(""),
            Lifecycle::Beta(num) => vec!["beta", &num.to_string()].join(""),
            Lifecycle::Stable(num) => vec!["stable", &num.to_string()].join(""),
        };
        vec![version, life_cycle].join("-")
    }
}

impl HDKVersion {
    pub fn new(version_string: &str) -> HcResult<HDKVersion> {
        let mut splits = version_string.split('-');
        let version = splits.next().ok_or("Could not get version")?;

        let mut version_splits = version.trim_start_matches('v').split('.');
        let versioning = (
            version_splits
                .next()
                .ok_or("Could not get version")?
                .parse::<i16>()
                .map_err(|_| HolochainError::ErrorGeneric("Could not parse version".to_string()))?,
            version_splits
                .next()
                .ok_or("Could not get version")?
                .parse::<i16>()
                .map_err(|_| HolochainError::ErrorGeneric("Could not parse version".to_string()))?,
            version_splits
                .next()
                .ok_or("Could not get version")?
                .parse::<i16>()
                .map_err(|_| HolochainError::ErrorGeneric("Could not parse version".to_string()))?,
        );

        let lifecycle = get_lifecycle(splits.next().ok_or("Could not get lifecycle")?)?;

        Ok(HDKVersion {
            versioning,
            lifecycle,
        })
    }
}
