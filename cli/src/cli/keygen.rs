use error::DefaultResult;
use holochain_common::paths::keys_directory;
use holochain_dpki::{
    bundle::KeyBundle,
    keypair::{Keypair, SEEDSIZE},
    util::PwHashConfig,
};
use holochain_sodium::{pwhash, random::random_secbuf, secbuf::SecBuf};
use rpassword;
use std::{
    fs::{create_dir_all, File},
    io::prelude::*,
    path::PathBuf,
};

pub fn keygen(path: Option<PathBuf>, passphrase: Option<String>) -> DefaultResult<()> {
    let passphrase = passphrase
        .unwrap_or_else(|| rpassword::read_password_from_tty(Some("Passphrase: ")).unwrap());

    let mut seed = SecBuf::with_secure(SEEDSIZE);
    random_secbuf(&mut seed);
    let mut keypair = Keypair::new_from_seed(&mut seed).unwrap();
    let passphrase_bytes = passphrase.as_bytes();
    let mut passphrase_buf = SecBuf::with_insecure(passphrase_bytes.len());
    passphrase_buf
        .write(0, passphrase_bytes)
        .expect("SecBuf must be writeable");

    let bundle: KeyBundle = keypair
        .get_bundle(
            &mut passphrase_buf,
            "hint".to_string(),
            Some(PwHashConfig(
                pwhash::OPSLIMIT_INTERACTIVE,
                pwhash::MEMLIMIT_INTERACTIVE,
                pwhash::ALG_ARGON2ID13,
            )),
        )
        .unwrap();

    let path = if None == path {
        let p = keys_directory();
        create_dir_all(p.clone())?;
        p.join(keypair.pub_keys.clone())
    } else {
        path.unwrap()
    };

    let mut file = File::create(path.clone())?;
    file.write_all(serde_json::to_string(&bundle).unwrap().as_bytes())?;
    println!("Agent keys with public address: {}", keypair.pub_keys);
    println!("written to: {}.", path.to_str().unwrap());
    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use holochain_dpki::bundle::KeyBundle;
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

        let bundle: KeyBundle = serde_json::from_str(&contents).unwrap();
        let mut passphrase = SecBuf::with_insecure_from_string(passphrase);
        let keypair = Keypair::from_bundle(
            &bundle,
            &mut passphrase,
            Some(PwHashConfig(
                pwhash::OPSLIMIT_INTERACTIVE,
                pwhash::MEMLIMIT_INTERACTIVE,
                pwhash::ALG_ARGON2ID13,
            )),
        );

        assert!(keypair.is_ok());

        let _ = remove_file(path);
    }
}
