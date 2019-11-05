use holochain_core_types::error::HolochainError;
use reqwest::{self, Url};
use serde::{
    de::{self, Deserializer, Visitor},
    ser::{self, Serialize, Serializer},
    Deserialize,
};
use std::{fmt, fs::File, io::Read, path::PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum DnaLocation {
    File(PathBuf),
    /// http or https only
    Url(Url),
}

impl DnaLocation {
    pub fn get_content(&self) -> Result<String, HolochainError> {
        match self {
            DnaLocation::File(file) => {
                let mut f = File::open(file)?;
                let mut content = String::new();
                f.read_to_string(&mut content)?;
                Ok(content)
            }
            DnaLocation::Url(url) => {
                let content: String = reqwest::get::<Url>(url.clone())
                    .map_err(|e| HolochainError::ErrorGeneric(format!("request failed: {}", e)))?
                    .text()
                    .map_err(|e| {
                        HolochainError::ErrorGeneric(format!("could not get text response: {}", e))
                    })?;
                debug!("Finished downloading file from {}", url);
                Ok(content)
            }
        }
    }
}

impl fmt::Display for DnaLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DnaLocation::File(file) => write!(f, "{}", file.to_string_lossy()),
            DnaLocation::Url(url) => write!(f, "{}", url),
        }
    }
}

impl From<PathBuf> for DnaLocation {
    fn from(path: PathBuf) -> Self {
        DnaLocation::File(path)
    }
}

impl From<Url> for DnaLocation {
    fn from(url: Url) -> Self {
        DnaLocation::Url(url)
    }
}

impl Serialize for DnaLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            DnaLocation::Url(url) => url.as_str(),
            DnaLocation::File(path) => path
                .to_str()
                .ok_or_else(|| ser::Error::custom(format!("invalid PathBuf: {:?}", path)))?,
        })
    }
}

struct DnaLocationVisitor;

impl<'de> Visitor<'de> for DnaLocationVisitor {
    type Value = DnaLocation;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing a file path or a URL")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Url::parse(value)
            // NB: we throw away this error, it is only to appease the type checker
            .map_err(|e| de::Error::custom(format!("Not a URL: {} {}", value, e)))
            .and_then(|url| match url.scheme() {
                "http" | "https" => Ok(DnaLocation::Url(url)),
                scheme => Err(de::Error::custom(format!(
                    "Unsupported URL scheme for DNA: {}",
                    scheme
                ))),
            })
            .or_else(|_: E| Ok(PathBuf::from(value).into()))
    }
}

impl<'de> Deserialize<'de> for DnaLocation {
    fn deserialize<D>(deserializer: D) -> Result<DnaLocation, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DnaLocationVisitor)
    }
}

#[cfg(test)]
mod tests {

    use super::{DnaLocation, Url};
    use serde_json;
    use std::path::PathBuf;

    #[test]
    fn serialize_dna_location() {
        let url = DnaLocation::Url(Url::parse("http://holochain.love").unwrap());
        let file = DnaLocation::File(PathBuf::from("some/path.dna.json"));
        assert_eq!(
            serde_json::to_string(&url).unwrap(),
            r#""http://holochain.love/""#
        );
        assert_eq!(
            serde_json::to_string(&file).unwrap(),
            r#""some/path.dna.json""#
        );
    }

    #[test]
    fn deserialize_dna_location() {
        let location_url = serde_json::from_str(r#""http://holochain.love""#).unwrap();
        match location_url {
            DnaLocation::Url(url) => assert_eq!(url.to_string(), "http://holochain.love/"),
            _ => panic!("Expected URL"),
        };
        let location_file = serde_json::from_str(r#""C:\\dnas\\foo.dna.json""#).unwrap();
        match location_file {
            DnaLocation::File(path) => assert_eq!(path.to_str().unwrap(), "C:\\dnas\\foo.dna.json"),
            _ => panic!(format!("Expected File, got: {:?}", location_file)),
        };
    }
}
