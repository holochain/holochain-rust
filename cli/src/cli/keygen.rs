use error::DefaultResult;
use holochain_common::paths::keys_directory;
use holochain_conductor_api::{key_loaders::mock_passphrase_manager, keystore::Keystore};
use holochain_dpki::{key_bundle::KeyBundle, utils::SeedContext, AGENT_ID_CTX, SEED_SIZE};
use holochain_sodium::secbuf::SecBuf;
use rpassword;
use std::{fs::create_dir_all, path::PathBuf};

pub fn keygen(
    agent_name: &str,
    path: Option<PathBuf>,
    passphrase: Option<String>,
) -> DefaultResult<()> {
    println!(
        "This will create a new agent keystore - that is all keys needed to represent one agent."
    );
    println!("This keystore will be stored in a file, encrypted with a passphrase.");
    println!("The passphrase is securing the keys and will be needed, together with the file, in order to use the key.");
    println!("Please enter a secret passphrase below, you will have to enter it again when unlocking this keystore to use within a Holochain conductor.");

    let passphrase = passphrase.unwrap_or_else(|| {
        let passphrase1 = rpassword::read_password_from_tty(Some("Passphrase: ")).unwrap();
        let passphrase2 = rpassword::read_password_from_tty(Some("Reenter Passphrase: ")).unwrap();
        if passphrase1 != passphrase2 {
            println!("Passphrases do not match. Please retry...");
            ::std::process::exit(1);
        }
        passphrase1
    });

    let mut keystore = Keystore::new(mock_passphrase_manager(agent_name.to_owned()))?;
    keystore.add_random_seed("root_seed", SEED_SIZE)?;

    let context = SeedContext::new(AGENT_ID_CTX);
    let (pub_key, _) = keystore.add_keybundle_from_seed("root_seed", agent_name, &context, 1)?;

    let path = if None == path {
        let p = keys_directory();
        create_dir_all(p.clone())?;
        p.join(pub_key.clone())
    } else {
        path.unwrap()
    };

    keystore.save(path)?;

    println!("");
    println!("Succesfully created new agent keystore.");
    println!("");
    println!("Public address: {}", pub_key);
    println!("Bundle written to: {}.", path.to_str().unwrap());
    println!("");
    println!("You can set this file in a conductor config as key_file for an agent.");
    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use holochain_dpki::key_blob::KeyBlob;
    use std::{
        fs::{remove_file, File},
        path::PathBuf,
    };

    #[test]
    fn keygen_roundtrip() {
        let path = PathBuf::new().join("test.key");
        let passphrase = String::from("secret");

        keygen(
            "test-instance",
            Some(path.clone()),
            Some(passphrase.clone()),
        )
        .expect("Keygen should work");

        let mut file = File::open(path.clone()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let blob: KeyBlob = serde_json::from_str(&contents).unwrap();
        let mut passphrase = SecBuf::with_insecure_from_string(passphrase);
        let keybundle = KeyBundle::from_blob(&blob, &mut passphrase, None);

        assert!(keybundle.is_ok());

        let _ = remove_file(path);
    }
}
