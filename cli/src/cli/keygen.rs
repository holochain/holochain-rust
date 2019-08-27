use holochain_common::paths::keys_directory;
use holochain_conductor_api::{
    key_loaders::mock_passphrase_manager,
    keystore::{Keystore, PRIMARY_KEYBUNDLE_ID},
};
use holochain_core_types::error::HcResult;
use holochain_dpki::{
    seed::{SeedType, TypedSeed},
    utils::SeedContext,
    DEVICE_CTX,
};
use std::{
    fs::create_dir_all,
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use util::{get_secure_string_double_check, get_seed};

pub fn keygen(
    path: Option<PathBuf>,
    passphrase: Option<String>,
    quiet: bool,
    root_seed_mnemonic: String,
    device_derivation_index: Option<u64>,
) -> HcResult<()> {
    let passphrase = passphrase.unwrap_or_else(|| {
        if !quiet {
            println!(
                "
This will create a new agent keystore and populate it with an agent keybundle
containing a public and a private key, for signing and encryption by the agent.
This keybundle will be stored encrypted by passphrase within the keystore file.
The passphrase is securing the keys and will be needed, together with the file,
in order to use the key.
Please enter a secret passphrase below. You will have to enter it again
when unlocking the keybundle to use within a Holochain conductor."
            );
            print!("Passphrase: ");
            io::stdout().flush().expect("Could not flush stdout");
        }
        get_secure_string_double_check("Passphrase", quiet).expect("Could not retrieve passphrase")
    });

    if !quiet {
        println!("Generating keystore (this will take a few moments)...");
    }

    let (keystore, pub_key) = if let Some(derivation_index) = device_derivation_index {
        println!("This keystore is to be generated from a DPKI root seed.");
        keygen_dpki(root_seed_mnemonic, Some(passphrase.clone()), derivation_index, passphrase)?
    } else {
        keygen_standalone(passphrase)?
    };

    let path = if None == path {
        let p = keys_directory();
        create_dir_all(p.clone())?;
        p.join(pub_key.clone())
    } else {
        path.unwrap()
    };

    keystore.save(path.clone())?;
    let path_str = path.to_str().unwrap();

    if quiet {
        println!("{}", pub_key);
        println!("{}", path_str);
    } else {
        println!("");
        println!("Succesfully created new agent keystore.");
        println!("");
        println!("Public address: {}", pub_key);
        println!("Keystore written to: {}", path_str);
        println!("");
        println!("You can set this file in a conductor config as keystore_file for an agent.");
    }
    Ok(())
}

fn keygen_standalone(keystore_passphrase: String) -> HcResult<(Keystore, String)> {
    Keystore::new_standalone(mock_passphrase_manager(keystore_passphrase), None)
}

fn keygen_dpki(root_seed_mnemonic: String, root_seed_passphrase: Option<String>, derivation_index: u64, keystore_passphrase: String) -> HcResult<(Keystore, String)> {
    let mut root_seed = match get_seed(root_seed_mnemonic, root_seed_passphrase, SeedType::Root)? { TypedSeed::Root(s) => s, _ => unreachable!() };
    let mut keystore = Keystore::new(mock_passphrase_manager(keystore_passphrase), None)?;
    let device_seed = root_seed.generate_device_seed(&SeedContext::new(DEVICE_CTX), derivation_index)?;
    keystore.add("device_seed", Arc::new(Mutex::new(device_seed.into())))?;
    let (pub_key, _) = keystore.add_keybundle_from_seed("device_seed", PRIMARY_KEYBUNDLE_ID)?;
    Ok((keystore, pub_key))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use cli::dpki;
    use holochain_conductor_api::{
        key_loaders::mock_passphrase_manager,
        keystore::{Keystore, PRIMARY_KEYBUNDLE_ID},
    };
    use std::{fs::remove_file, path::PathBuf};

    #[test]
    fn keygen_roundtrip_no_dpki() {
        let path = PathBuf::new().join("test.key");
        let passphrase = String::from("secret");

        keygen(
            Some(path.clone()),
            Some(passphrase.clone()),
            true,
            String::from(""),
            None,
        )
        .expect("Keygen should work");

        let mut keystore =
            Keystore::new_from_file(path.clone(), mock_passphrase_manager(passphrase), None)
                .unwrap();

        let keybundle = keystore.get_keybundle(PRIMARY_KEYBUNDLE_ID);

        assert!(keybundle.is_ok());

        let _ = remove_file(path);
    }

    #[test]
    fn keygen_roundtrip_with_dpki() {
        let path = PathBuf::new().join("test_dpki.key");
        let passphrase = String::from("secret_dpki");

        let mnemonic = dpki::genroot(Some("dummy passphrase".to_string()))
            .expect("Could not generate root seed mneomonic");

        keygen(
            Some(path.clone()),
            Some(passphrase.clone()),
            true,
            mnemonic,
            Some(1),
        )
        .expect("Keygen should work");

        let mut keystore =
            Keystore::new_from_file(path.clone(), mock_passphrase_manager(passphrase), None)
                .unwrap();

        let keybundle = keystore.get_keybundle(PRIMARY_KEYBUNDLE_ID);

        assert!(keybundle.is_ok());

        let _ = remove_file(path);
    }
}
