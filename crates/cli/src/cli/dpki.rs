use crate::{
    cli::keygen,
    util::{self, get_secure_string_double_check, user_prompt, user_prompt_yes_no, WordCountable},
};
use holochain_core_types::error::HcResult;
use holochain_dpki::{
    key_bundle::KeyBundle,
    keypair::KeyPair,
    seed::{MnemonicableSeed, RootSeed, SeedTrait, SeedType, TypedSeed},
    utils::generate_random_seed_buf,
};
use lib3h_sodium::secbuf::SecBuf;
use std::{path::PathBuf, str::FromStr, string::ParseError};
use structopt::StructOpt;

const MNEMONIC_WORD_COUNT: usize = 24;
const ENCRYPTED_MNEMONIC_WORD_COUNT: usize = 2 * MNEMONIC_WORD_COUNT;

const DEFAULT_REVOCATION_KEY_DEV_INDEX: u64 = 1;
const DEFAULT_AUTH_KEY_DEV_INDEX: u64 = 1;

pub enum SignType {
    Revoke,
    Auth,
}

impl FromStr for SignType {
    type Err = ParseError;
    fn from_str(day: &str) -> Result<Self, Self::Err> {
        match day {
            "revoke" => Ok(SignType::Revoke),
            "auth" => Ok(SignType::Auth),
            _ => panic!(),
        }
    }
}

#[derive(StructOpt)]
pub enum Dpki {
    #[structopt(
        name = "genroot",
        about = "Generate a new random DPKI root seed. This is encrypyed with a passphrase and printed in BIP39 mnemonic form to stdout. Both the passphrase and mnemonic should be recorded and kept safe to be used later for key management."
    )]
    GenRoot {
        passphrase: Option<String>,
        quiet: bool,
    },

    #[structopt(
        name = "keygen",
        about = "Identical to `hc keygen` but derives agent key pair from a DPKI root seed at a given derivation index. This allows the keys to be recovered provided the root seed is known."
    )]
    Keygen {
        #[structopt(long, short, help = "Specify path of file")]
        path: Option<PathBuf>,

        #[structopt(
            long,
            short,
            help = "Only print machine-readable output; intended for use by programs and scripts"
        )]
        quiet: bool,

        #[structopt(
            long,
            short,
            help = "Set passphrase via argument and don't prompt for it (not reccomended)"
        )]
        keystore_passphrase: Option<String>,

        #[structopt(
            long,
            short,
            help = "Use insecure, hard-wired passphrase for testing and Don't ask for passphrase"
        )]
        nullpass: bool,

        #[structopt(
            long,
            short,
            help = "Set root seed via argument and don't prompt for it (not reccomended). BIP39 mnemonic encoded root seed to derive device seed and agent key from"
        )]
        root_seed: Option<String>,

        #[structopt(
            long,
            short,
            help = "Set mnemonic passphrase via argument and don't prompt for it (not reccomended)"
        )]
        mnemonic_passphrase: Option<String>,

        #[structopt(help = "Derive device seed from root seed with this index")]
        device_derivation_index: u64,
    },

    #[structopt(
        name = "genrevoke",
        about = "Generate a revocation seed given an encrypted root seed mnemonic, passphrase and derivation index."
    )]
    GenRevoke {
        #[structopt(help = "Derive revocation seed from root seed with this index")]
        derivation_index: u64,

        #[structopt(
            long,
            short,
            help = "unsecurely pass passphrase to decrypt root seed (not reccomended). Will prompt if encrypted seed provided."
        )]
        root_seed_passphrase: Option<String>,

        #[structopt(
            long,
            short,
            help = "unsecurely pass passphrase to encrypt revocation seed (not reccomended)."
        )]
        revocation_seed_passphrase: Option<String>,

        #[structopt(
            long,
            short,
            help = "Only print machine-readable output; intended for use by programs and scripts"
        )]
        quiet: bool,
    },

    #[structopt(
        name = "genauth",
        about = "Generate an auth seed given an encrypted root seed mnemonic, passphrase and derivation index."
    )]
    GenAuth {
        #[structopt(help = "Derive auth seed from root seed with this index")]
        derivation_index: u64,

        #[structopt(
            long,
            short,
            help = "unsecurely pass passphrase to decrypt root seed (not reccomended). Will prompt if encrypted seed provided."
        )]
        root_seed_passphrase: Option<String>,

        #[structopt(
            long,
            short,
            help = "unsecurely pass passphrase to encrypt auth seed (not reccomended)."
        )]
        auth_seed_passphrase: Option<String>,

        #[structopt(
            long,
            short,
            help = "Only print machine-readable output; intended for use by programs and scripts"
        )]
        quiet: bool,
    },

    #[structopt(
        name = "sign",
        about = "Produce the signed string needed to revoke a key given a revocation seed mnemonic and passphrase."
    )]
    Sign {
        #[structopt(
            help = "Public key to revoke/authorize (or any other string you want to sign with an auth/revocation key)"
        )]
        key: String,

        #[structopt(
            long,
            short,
            help = "unsecurely pass passphrase to decrypt revocation seed (not reccomended). Will prompt if encrypted seed provided."
        )]
        passphrase: Option<String>,

        #[structopt(
            long,
            short,
            help = "Only print machine-readable output; intended for use by programs and scripts"
        )]
        quiet: bool,

        #[structopt(long, short, help = "How to interpred seed (revoke/auth)")]
        sign_type: SignType,
    },
}

impl Dpki {
    pub fn execute(self) -> HcResult<String> {
        match self {
            Self::GenRoot { passphrase, quiet } => genroot(passphrase, quiet),
            Self::Keygen {
                path,
                keystore_passphrase,
                nullpass,
                quiet,
                root_seed,
                mnemonic_passphrase,
                device_derivation_index,
            } => keygen(
                path,
                keystore_passphrase,
                nullpass,
                mnemonic_passphrase,
                root_seed,
                Some(device_derivation_index),
                quiet,
            )
            .map(|_| "success".to_string()),
            Self::GenRevoke {
                derivation_index,
                root_seed_passphrase,
                revocation_seed_passphrase,
                quiet,
            } => genrevoke(
                root_seed_passphrase,
                revocation_seed_passphrase,
                derivation_index,
                quiet,
            ),
            Self::GenAuth {
                derivation_index,
                root_seed_passphrase,
                auth_seed_passphrase,
                quiet,
            } => genauth(
                root_seed_passphrase,
                auth_seed_passphrase,
                derivation_index,
                quiet,
            ),
            Self::Sign {
                passphrase,
                key,
                sign_type,
                quiet,
            } => sign(passphrase, key, sign_type, quiet),
        }
    }
}

fn genroot(passphrase: Option<String>, quiet: bool) -> HcResult<String> {
    user_prompt(
        "This will generate a new random DPKI root seed.
You should only have to do this once and you should keep the seed safe.
It will be printed out once as a mnemonic at the end of this process.
The root seed can be used to generate new device, revocation and auth keys.\n",
        quiet,
    );

    let passphrase = passphrase.or_else(|| {
        match user_prompt_yes_no("Would you like to encrypt the root seed?", quiet) {
            true => Some(
                get_secure_string_double_check("Root Seed Passphrase", quiet)
                    .expect("Could not read revocation passphrase"),
            ),
            false => None,
        }
    });
    println!();
    genroot_inner(passphrase)
}

pub(crate) fn genroot_inner(passphrase: Option<String>) -> HcResult<String> {
    let seed_buf = generate_random_seed_buf();
    let mut root_seed = RootSeed::new(seed_buf);
    match passphrase {
        Some(passphrase) => root_seed.encrypt(passphrase, None)?.get_mnemonic(),
        None => root_seed.seed_mut().get_mnemonic(),
    }
}

fn genrevoke(
    root_seed_passphrase: Option<String>,
    revocation_seed_passphrase: Option<String>,
    derivation_index: u64,
    quiet: bool,
) -> HcResult<String> {
    user_prompt(
        "This will generate a new revocation seed derived from a root seed.
This can be used to revoke access to keys you have previously authorized.\n",
        quiet,
    );

    let root_seed_mnemonic = get_secure_string_double_check("Root Seed", quiet)?;
    let root_seed_passphrase = match root_seed_mnemonic.word_count() {
        MNEMONIC_WORD_COUNT => None, // ignore any passphrase passed if it is an unencrypted mnemonic
        ENCRYPTED_MNEMONIC_WORD_COUNT => root_seed_passphrase.or_else(|| {
            Some(
                get_secure_string_double_check("Root Seed Passphrase", quiet)
                    .expect("Could not read passphrase"),
            )
        }),
        _ => panic!("Invalid word count for mnemonic"),
    };
    let revocation_seed_passphrase =
        revocation_seed_passphrase.or_else(|| {
            match user_prompt_yes_no("Would you like to encrypt the revocation seed?", quiet) {
                true => Some(
                    get_secure_string_double_check("Revocation Seed Passphrase", quiet)
                        .expect("Could not read revocation passphrase"),
                ),
                false => None,
            }
        });
    println!();
    let (mnemonic, pubkey) = genauth_genrevoke_inner(
        root_seed_mnemonic,
        root_seed_passphrase,
        revocation_seed_passphrase,
        derivation_index,
        SignType::Revoke,
    )?;
    Ok(format!("Public Key: {}\n\nMnemonic: {}", pubkey, mnemonic))
}

fn genauth(
    root_seed_passphrase: Option<String>,
    auth_seed_passphrase: Option<String>,
    derivation_index: u64,
    quiet: bool,
) -> HcResult<String> {
    user_prompt(
        "This will generate a new authorization seed derived from a root seed.
This can be used to authorize new keys in DPKI.\n",
        quiet,
    );

    let root_seed_mnemonic = get_secure_string_double_check("Root Seed", quiet)?;
    let root_seed_passphrase = match root_seed_mnemonic.word_count() {
        MNEMONIC_WORD_COUNT => None, // ignore any passphrase passed if it is an unencrypted mnemonic
        ENCRYPTED_MNEMONIC_WORD_COUNT => root_seed_passphrase.or_else(|| {
            Some(
                get_secure_string_double_check("Root Seed Passphrase", quiet)
                    .expect("Could not read passphrase"),
            )
        }),
        _ => panic!("Invalid word count for mnemonic"),
    };
    let auth_seed_passphrase = auth_seed_passphrase.or_else(|| {
        match user_prompt_yes_no("Would you like to encrypt the auth seed?", quiet) {
            true => Some(
                get_secure_string_double_check("Auth Seed Passphrase", quiet)
                    .expect("Could not read auth passphrase"),
            ),
            false => None,
        }
    });
    println!();
    let (mnemonic, pubkey) = genauth_genrevoke_inner(
        root_seed_mnemonic,
        root_seed_passphrase,
        auth_seed_passphrase,
        derivation_index,
        SignType::Auth,
    )?;
    Ok(format!("Public Key: {}\n\nMnemonic: {}", pubkey, mnemonic))
}

fn genauth_genrevoke_inner(
    root_seed_mnemonic: String,
    root_seed_passphrase: Option<String>,
    new_seed_passphrase: Option<String>,
    derivation_index: u64,
    key_type: SignType,
) -> HcResult<(String, String)> {
    let mut root_seed =
        match util::get_seed(root_seed_mnemonic, root_seed_passphrase, SeedType::Root)? {
            TypedSeed::Root(s) => s,
            _ => unreachable!(),
        };

    match key_type {
        SignType::Revoke => {
            let mut revocation_seed = root_seed.generate_revocation_seed(derivation_index)?;
            let pubkey = revocation_seed
                .generate_revocation_key(DEFAULT_REVOCATION_KEY_DEV_INDEX)?
                .sign_keys
                .public();
            match new_seed_passphrase {
                Some(passphrase) => Ok((
                    revocation_seed.encrypt(passphrase, None)?.get_mnemonic()?,
                    pubkey,
                )),
                None => Ok((revocation_seed.seed_mut().get_mnemonic()?, pubkey)),
            }
        }
        SignType::Auth => {
            // TODO: Allow different derivation paths for the auth key
            let mut auth_seed = root_seed
                .generate_device_seed(derivation_index)?
                .generate_auth_seed(1)?;
            let pubkey = auth_seed
                .generate_auth_key(DEFAULT_AUTH_KEY_DEV_INDEX)?
                .sign_keys
                .public();
            match new_seed_passphrase {
                Some(passphrase) => {
                    Ok((auth_seed.encrypt(passphrase, None)?.get_mnemonic()?, pubkey))
                }
                None => Ok((auth_seed.seed_mut().get_mnemonic()?, pubkey)),
            }
        }
    }
}

fn sign(
    passphrase: Option<String>,
    key_string: String,
    sign_type: SignType,
    quiet: bool,
) -> HcResult<String> {
    user_prompt("This will sign a given key/string with a auth/revocation key.
The resulting signed message can be used to publish a DPKI auth/revocation message which will auth/revoke a key.\n", quiet);

    let seed_mnemonic = get_secure_string_double_check("Seed", false)?;
    let passphrase = match seed_mnemonic.word_count() {
        MNEMONIC_WORD_COUNT => None, // ignore any passphrase passed if it is an unencrypted mnemonic
        ENCRYPTED_MNEMONIC_WORD_COUNT => passphrase.or_else(|| {
            Some(
                get_secure_string_double_check("Seed Passphrase", quiet)
                    .expect("Could not read passphrase"),
            )
        }),
        _ => panic!("Invalid word count for mnemonic"),
    };
    println!();
    sign_inner(seed_mnemonic, passphrase, key_string, sign_type)
}

fn sign_inner(
    seed_mnemonic: String,
    passphrase: Option<String>,
    key_string: String,
    sign_type: SignType,
) -> HcResult<String> {
    let mut keypair = match sign_type {
        SignType::Revoke => {
            let mut revocation_seed =
                match util::get_seed(seed_mnemonic, passphrase, SeedType::Revocation)? {
                    TypedSeed::Revocation(s) => s,
                    _ => unreachable!(),
                };
            revocation_seed.generate_revocation_key(DEFAULT_REVOCATION_KEY_DEV_INDEX)?
        }
        SignType::Auth => {
            let mut auth_seed = match util::get_seed(seed_mnemonic, passphrase, SeedType::Auth)? {
                TypedSeed::Auth(s) => s,
                _ => unreachable!(),
            };
            auth_seed.generate_auth_key(DEFAULT_AUTH_KEY_DEV_INDEX)?
        }
    };
    sign_with_key_from_seed(&mut keypair, key_string)
}

fn sign_with_key_from_seed(keypair: &mut KeyBundle, key_string: String) -> HcResult<String> {
    let mut data_buf = SecBuf::with_insecure_from_string(key_string);
    let mut signature_buf = keypair.sign(&mut data_buf)?;
    let buf = signature_buf.read_lock();
    let signature_str = base64::encode(&**buf);
    Ok(signature_str)
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use holochain_core_types::signature::{Provenance, Signature};
    use holochain_dpki::{keypair::*, utils::Verify};
    use holochain_persistence_api::cas::content::Address;

    #[test]
    fn can_verify_signature() {
        let payload = "test signing payload";
        let mut seed = generate_random_seed_buf();
        let sign_keys = SigningKeyPair::new_from_seed(&mut seed).unwrap();
        let enc_keys = EncryptingKeyPair::new_from_seed(&mut seed).unwrap();
        let mut key_bundle = KeyBundle::new(sign_keys, enc_keys).unwrap();

        let sig = sign_with_key_from_seed(&mut key_bundle, payload.to_string()).unwrap();

        let prov = Provenance::new(
            Address::from(key_bundle.sign_keys.public),
            Signature::from(sig),
        );
        assert!(prov.verify(payload.to_string()).unwrap());
    }
}
