use crate::{
    config::DnaConfiguration,
    container::Container,
};
use holochain_core_types::{
    cas::content::AddressableContent,
    error::HolochainError
};
use std::{
    path::PathBuf,
    sync::Arc,
};

pub trait ContainerAdmin {
    fn install_dna_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError>;
    fn uninstall_dna(&mut self, id: String) -> Result<(), HolochainError>;
}

impl ContainerAdmin for Container {
    fn install_dna_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError> {
        let path_string = path.to_str().ok_or(HolochainError::ConfigError("invalid path".into()))?;
        let dna = Arc::get_mut(&mut self.dna_loader).unwrap()(&path_string.into()).map_err(
            |e| {
                HolochainError::ConfigError(format!(
                    "Could not load DNA file \"{}\", Error: {}",
                    path_string,
                    e.to_string()
                ))
            },
        )?;

        let new_dna = DnaConfiguration {
            id: id.clone(),
            file: path_string.into(),
            hash: dna.address().to_string(),
        };
        self.config.dnas.push(new_dna);
        self.save_config()?;
        println!("Installed DNA from {} as \"{}\"", path_string, id);
        Ok(())
    }

    fn uninstall_dna(&mut self, _id: String) -> Result<(), HolochainError> {
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::{Configuration, load_configuration};
    use crate::container::{DnaLoader, tests::example_dna_string};
    use holochain_core_types::{dna::Dna, json::{JsonString}};
    use std::{
        convert::TryFrom,
        fs::File,
        io::Read,
    };

    pub fn test_dna_loader() -> DnaLoader {
        let loader = Box::new(|_: &String| {
            Ok(Dna::try_from(JsonString::from(example_dna_string())).unwrap())
        }) as Box<FnMut(&String) -> Result<Dna, HolochainError> + Send + Sync>;
        Arc::new(loader)
    }

    pub fn test_toml() -> String {
        r#"bridges = []
interfaces = []

[[agents]]
id = "test-agent-1"
key_file = "holo_tester.key"
name = "Holo Tester 1"
public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB"

[[dnas]]
file = "app_spec.hcpkg"
hash = "Qm328wyq38924y"
id = "test-dna"

[[instances]]
agent = "test-agent-1"
dna = "test-dna"
id = "test-instance-1"

[instances.storage]
type = "memory"

[logger]
type = ""
[[logger.rules.rules]]
color = "red"
exclude = false
pattern = "^err/"

[[logger.rules.rules]]
color = "white"
exclude = false
pattern = "^debug/dna"

[[logger.rules.rules]]
exclude = false
pattern = ".*"
    "#
            .to_string()
    }

    #[test]
    fn test_install_dna_from_file() {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let mut container = Container::from_config(config.clone());
        container.dna_loader = test_dna_loader();
        container.load_config().unwrap();

        let mut tmp_config_path = PathBuf::new();
        tmp_config_path.push("tmp-test-container-config.toml");
        container.set_config_path(tmp_config_path.clone());

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.hcpkg");

        assert_eq!(
            container.install_dna_from_file(new_dna_path.clone(), String::from("new-dna")),
            Ok(()),
        );

        let new_dna = Arc::get_mut(&mut test_dna_loader()).unwrap()(&String::from("new-dna.hcpkg")).unwrap();

        assert_eq!(
            container.config().dnas.len(),
            2,
        );
        assert_eq!(
            container.config().dnas,
            vec![
                DnaConfiguration {
                    id: String::from("test-dna"),
                    file: String::from("app_spec.hcpkg"),
                    hash: String::from("Qm328wyq38924y"),
                },
                DnaConfiguration {
                    id: String::from("new-dna"),
                    file: String::from("new-dna.hcpkg"),
                    hash: String::from(new_dna.address()),
                },
            ]
        );

        let mut config_contents = String::new();
        let mut file = File::open(&tmp_config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents).expect("Could not read temp config file");

        assert_eq!(
            config_contents,
r#"bridges = []
interfaces = []

[[agents]]
id = "test-agent-1"
key_file = "holo_tester.key"
name = "Holo Tester 1"
public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB"

[[dnas]]
file = "app_spec.hcpkg"
hash = "Qm328wyq38924y"
id = "test-dna"

[[dnas]]
file = "new-dna.hcpkg"
hash = "QmPB7PJUjwj6zap7jB7oyk616sCRSSnNFRNouqhit6kMTr"
id = "new-dna"

[[instances]]
agent = "test-agent-1"
dna = "test-dna"
id = "test-instance-1"

[instances.storage]
type = "memory"

[logger]
type = ""
[[logger.rules.rules]]
color = "red"
exclude = false
pattern = "^err/"

[[logger.rules.rules]]
color = "white"
exclude = false
pattern = "^debug/dna"

[[logger.rules.rules]]
exclude = false
pattern = ".*"
"#
        );
    }
}
