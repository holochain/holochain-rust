use error::DefaultResult;
use holochain_common::paths::keys_directory;
use holochain_conductor_api::{
    key_loaders::mock_passphrase_manager,
    keystore::{Keystore, PRIMARY_KEYBUNDLE_ID},
};
use rpassword;
use std::{
    fs::create_dir_all,
    io::{self, Write},
    path::PathBuf,
};

pub fn keygen(path: Option<PathBuf>, passphrase: Option<String>) -> DefaultResult<()> {
    println!("This will create a new agent keystore and populate it with an agent keybundle");
    println!("(=all keys needed to represent an agent: public/private keys for signing/encryption");
    println!("This keybundle will be stored encrypted by passphrase within the keystore file.");
    println!("The passphrase is securing the keys and will be needed, together with the file, in order to use the key.");
    println!("Please enter a secret passphrase below, you will have to enter it again when unlocking these keys to use within a Holochain conductor.");

    let passphrase = passphrase.unwrap_or_else(|| {
        print!("Passphrase: ");
        io::stdout().flush().expect("Could not flush stdout");
        let passphrase1 = rpassword::read_password().unwrap();
        print!("Re-enter passphrase: ");
        io::stdout().flush().expect("Could not flush stdout");
        let passphrase2 = rpassword::read_password().unwrap();
        if passphrase1 != passphrase2 {
            println!("Passphrases do not match. Please retry...");
            ::std::process::exit(1);
        }
        passphrase1
    });

    let (keystore, pub_key) = Keystore::new_standalone(mock_passphrase_manager(passphrase), None)?;

    let path = if None == path {
        let p = keys_directory();
        create_dir_all(p.clone())?;
        p.join(pub_key.clone())
    } else {
        path.unwrap()
    };

    keystore.save(path.clone())?;

    println!("");
    println!("Succesfully created new agent keystore.");
    println!("");
    println!("Public address: {}", pub_key);
    println!("Keystore written to: {}", path.to_str().unwrap());
    println!("");
    println!("You can set this file in a conductor config as keystore_file for an agent.");
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

        keygen(Some(path.clone()), Some(passphrase.clone())).expect("Keygen should work");

        let mut keystore =
            Keystore::new_from_file(path.clone(), mock_passphrase_manager(passphrase), None)
                .unwrap();

        let keybundle = keystore.get_keybundle(PRIMARY_KEYBUNDLE_ID);

        assert!(keybundle.is_ok());

        let _ = remove_file(path);
    }
}
