use error::DefaultResult;
use holochain_common::paths::keys_directory;
use holochain_conductor_api::{
    key_loaders::mock_passphrase_manager,
    keystore::{Keystore, PRIMARY_KEYBUNDLE_ID},
};
use holochain_dpki::SEED_SIZE;
use rpassword;
use std::{
    fs::create_dir_all,
    io::{self, Write},
    path::PathBuf,
};

pub fn keygen(path: Option<PathBuf>, passphrase: Option<String>, quiet: bool) -> DefaultResult<()> {
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
        let passphrase1 = rpassword::read_password().unwrap();
        if !quiet {
            print!("Re-enter passphrase: ");
            io::stdout().flush().expect("Could not flush stdout");
        }
        let passphrase2 = rpassword::read_password().unwrap();
        if passphrase1 != passphrase2 {
            println!("Passphrases do not match. Please retry...");
            ::std::process::exit(1);
        }
        passphrase1
    });

    if !quiet {
        println!("Generating keystore (this will take a few moments)...");
    }

    let mut keystore = Keystore::new(mock_passphrase_manager(passphrase), None)?;
    keystore.add_random_seed("root_seed", SEED_SIZE)?;

    let (pub_key, _) = keystore.add_keybundle_from_seed("root_seed", PRIMARY_KEYBUNDLE_ID)?;

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

#[cfg(test)]
pub mod test {
    use super::*;
    use holochain_conductor_api::{key_loaders::mock_passphrase_manager, keystore::Keystore};
    use std::{fs::remove_file, path::PathBuf};

    #[test]
    fn keygen_roundtrip() {
        let path = PathBuf::new().join("test.key");
        let passphrase = String::from("secret");

        keygen(Some(path.clone()), Some(passphrase.clone()), true).expect("Keygen should work");

        let mut keystore =
            Keystore::new_from_file(path.clone(), mock_passphrase_manager(passphrase), None)
                .unwrap();

        let keybundle = keystore.get_keybundle(PRIMARY_KEYBUNDLE_ID);

        assert!(keybundle.is_ok());

        let _ = remove_file(path);
    }
}
