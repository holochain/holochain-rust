use error::DefaultResult;
use holochain_common::paths::keys_directory;
use holochain_conductor_lib::{
    key_loaders::mock_passphrase_manager,
    keystore::{Keystore, PRIMARY_KEYBUNDLE_ID},
};
use holochain_core_types::error::HcResult;
use holochain_dpki::seed::{SeedType, TypedSeed};
use holochain_locksmith::Mutex;
use std::{
    fs::create_dir_all,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};
use util::{get_secure_string_double_check, get_seed, user_prompt};

pub fn keygen(
    path: Option<PathBuf>,
    keystore_passphrase: Option<String>,
    nullpass: bool,
    mnemonic_passphrase: Option<String>,
    root_seed_mnemonic: Option<String>,
    device_derivation_index: Option<u64>,
    quiet: bool,
) -> HcResult<()> {
    user_prompt(
        "This will create a new agent keystore and populate it with an agent keybundle
containing a public and a private key, for signing and encryption by the agent.
This keybundle will be stored encrypted by passphrase within the keystore file.
The passphrase is securing the keys and will be needed, together with the file,
in order to use the key.\n",
        quiet,
    );

    let keystore_passphrase = match (keystore_passphrase, nullpass) {
        (None, true) => String::from(holochain_common::DEFAULT_PASSPHRASE),
        (Some(s), false) => s,
        (Some(_), true) => panic!(
            "Invalid combination of args. Cannot pass --nullpass and also provide a passphrase"
        ),
        (None, false) => {
            // prompt for the passphrase
            user_prompt(
                "Please enter a secret passphrase below. You will have to enter it again
when unlocking the keybundle to use within a Holochain conductor.\n",
                quiet,
            );
            io::stdout().flush().expect("Could not flush stdout");
            get_secure_string_double_check("keystore Passphrase", quiet)
                .expect("Could not retrieve passphrase")
        }
    };

    let (keystore, pub_key) = if let Some(derivation_index) = device_derivation_index {
        user_prompt("This keystore is to be generated from a DPKI root seed. You can regenerate this keystore at any time by using the same root key mnemonic and device derivation index.", quiet);

        let root_seed_mnemonic = root_seed_mnemonic.unwrap_or_else(|| {
            get_secure_string_double_check("Root Seed Mnemonic", quiet)
                .expect("Could not retrieve mnemonic")
        });

        match root_seed_mnemonic.split(' ').count() {
            24 => {
                // unencrypted mnemonic
                user_prompt(
                    "Generating keystore (this will take a few moments)...",
                    quiet,
                );
                keygen_dpki(
                    root_seed_mnemonic,
                    None,
                    derivation_index,
                    keystore_passphrase,
                )?
            }
            48 => {
                // encrypted mnemonic
                let mnemonic_passphrase = mnemonic_passphrase.unwrap_or_else(|| {
                    get_secure_string_double_check("Root Seed Mnemonic passphrase", quiet)
                        .expect("Could not retrieve mnemonic passphrase")
                });
                user_prompt(
                    "Generating keystore (this will take a few moments)...",
                    quiet,
                );
                keygen_dpki(
                    root_seed_mnemonic,
                    Some(mnemonic_passphrase),
                    derivation_index,
                    keystore_passphrase,
                )?
            }
            _ => panic!(
                "Invalid number of words in mnemonic. Must be 24 (unencrypted) or 48 (encrypted)"
            ),
        }
    } else {
        user_prompt(
            "Generating keystore (this will take a few moments)...",
            quiet,
        );
        keygen_standalone(keystore_passphrase)?
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

fn keygen_dpki(
    root_seed_mnemonic: String,
    root_seed_passphrase: Option<String>,
    derivation_index: u64,
    keystore_passphrase: String,
) -> HcResult<(Keystore, String)> {
    let mut root_seed = match get_seed(root_seed_mnemonic, root_seed_passphrase, SeedType::Root)? {
        TypedSeed::Root(s) => s,
        _ => unreachable!(),
    };
    let mut keystore = Keystore::new(mock_passphrase_manager(keystore_passphrase), None)?;
    let device_seed = root_seed.generate_device_seed(derivation_index)?;
    keystore.add("device_seed", Arc::new(Mutex::new(device_seed.into())))?;
    let (pub_key, _) = keystore.add_keybundle_from_seed("device_seed", PRIMARY_KEYBUNDLE_ID)?;
    Ok((keystore, pub_key))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use cli::dpki;
    use holochain_conductor_lib::{
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
            false,
            None,
            None,
            None,
            true,
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
        let keystore_passphrase = String::from("secret_dpki");
        let mnemonic_passphrase = String::from("dummy passphrase");

        let mnemonic = dpki::genroot_inner(Some(mnemonic_passphrase.clone()))
            .expect("Could not generate root seed mneomonic");

        keygen(
            Some(path.clone()),
            Some(keystore_passphrase.clone()),
            false,
            Some(mnemonic_passphrase),
            Some(mnemonic),
            Some(1),
            true,
        )
        .expect("Keygen should work");

        let mut keystore = Keystore::new_from_file(
            path.clone(),
            mock_passphrase_manager(keystore_passphrase),
            None,
        )
        .unwrap();

        let keybundle = keystore.get_keybundle(PRIMARY_KEYBUNDLE_ID);

        assert!(keybundle.is_ok());

        let _ = remove_file(path);
    }
}
