use error::DefaultResult;
use holochain_dpki::{bundle::KeyBundle, keypair::{Keypair, SEEDSIZE}, util::PwHashConfig};
use holochain_sodium::{pwhash, random::random_secbuf, secbuf::SecBuf};
use rpassword;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::fs::create_dir_all;

pub fn keygen() -> DefaultResult<()> {
    let passphrase = rpassword::read_password_from_tty(Some("Passphrase: ")).unwrap();

    let mut seed = SecBuf::with_secure(SEEDSIZE);
    random_secbuf(&mut seed);
    let mut keypair = Keypair::new_from_seed(&mut seed).unwrap();
    let passphrase_bytes = passphrase.as_bytes();
    let mut passphrase_buf = SecBuf::with_insecure(passphrase_bytes.len());
    passphrase_buf.write(0, passphrase_bytes).expect("SecBuf must be writeable");

    let bundle: KeyBundle = keypair
        .get_bundle(&mut passphrase_buf, "hint".to_string(), Some(PwHashConfig(
            pwhash::OPSLIMIT_INTERACTIVE,
            pwhash::MEMLIMIT_INTERACTIVE,
            pwhash::ALG_ARGON2ID13,
        )))
        .unwrap();

    let path = match directories::UserDirs::new() {
        Some(user_dirs) => user_dirs
            .home_dir()
            .join(".holochain")
            .join("keys"),
        None => PathBuf::new(),
    };

    create_dir_all(path.clone())?;
    let path = path.join(keypair.pub_keys);
    let mut file = File::create(path.clone())?;
    file.write_all(serde_json::to_string(&bundle).unwrap().as_bytes())?;
    println!("Wrote {}.", path.to_str().unwrap());
    Ok(())
}