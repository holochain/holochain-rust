use error::DefaultResult;
use holochain_common::paths::keys_directory;
use holochain_dpki::{
    key_bundle::KeyBundle,
    key_blob::Blobbable,
    seed::SeedType,
    SEED_SIZE,
};
use holochain_sodium::secbuf::SecBuf;
use rpassword;
use std::{
    fs::{create_dir_all, File},
    io::prelude::*,
    path::PathBuf,
};

pub fn keygen(path: Option<PathBuf>, passphrase: Option<String>) -> DefaultResult<()> {
    println!(
        "This will create a new agent key bundle - that is all keys needed to represent one agent."
    );
    println!("This key bundle will be stored in a file, encrypted with a passphrase.");
    println!("The passphrase is securing the keys and will be needed, together with the key file, in order to use the key.");
    println!("Please enter a secret passphrase below, you will have to enter it again when unlocking this key to use within a Holochain conductor.");

    let passphrase = passphrase.unwrap_or_else(|| {
        let passphrase1 = rpassword::read_password_from_tty(Some("Passphrase: ")).unwrap();
        let passphrase2 = rpassword::read_password_from_tty(Some("Reenter Passphrase: ")).unwrap();
        if passphrase1 != passphrase2 {
            println!("Passphrases do not match. Please retry...");
            ::std::process::exit(1);
        }
        passphrase1
    });

    let mut seed = SecBuf::with_secure(SEED_SIZE);
    seed.randomize();

    let mut keybundle = KeyBundle::new_from_seed_buf(&mut seed, SeedType::Mock)
        .expect("Failed to generate keybundle");
    let passphrase_bytes = passphrase.as_bytes();
    let mut passphrase_buf = SecBuf::with_insecure(passphrase_bytes.len());
    passphrase_buf
        .write(0, passphrase_bytes)
        .expect("SecBuf must be writeable");

    let blob = keybundle
        .as_blob(&mut passphrase_buf, "hint".to_string(), None)
        .expect("Failed to encrypt with passphrase.");

    let path = if None == path {
        let p = keys_directory();
        create_dir_all(p.clone())?;
        p.join(keybundle.get_id().clone())
    } else {
        path.unwrap()
    };

    let mut file = File::create(path.clone())?;
    file.write_all(serde_json::to_string(&blob).unwrap().as_bytes())?;
    println!("");
    println!("Succesfully created new agent keys.");
    println!("");
    println!("Public address: {}", keybundle.get_id());
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

        keygen(Some(path.clone()), Some(passphrase.clone())).expect("Keygen should work");

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
