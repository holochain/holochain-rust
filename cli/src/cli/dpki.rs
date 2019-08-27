use std::path::PathBuf;
use crate::util;
use crate::util::get_secure_string_double_check;
use crate::cli::keygen;
use holochain_core_types::error::HcResult;
use holochain_dpki::{
    key_bundle::KeyBundle,
    seed::{Seed, RootSeed, SeedTrait, TypedSeed, SeedType, MnemonicableSeed},
    utils::generate_random_seed_buf,
};
use structopt::StructOpt;
use lib3h_sodium::secbuf::SecBuf;


#[derive(StructOpt)]
pub enum Dpki {
    #[structopt(
        name = "genroot",
        about = "Generate a new random DPKI root seed. This is encrypyed with a passphrase and printed in BIP39 mnemonic form to stdout. Both the passphrase and mnemonic should be recorded and kept safe to be used later for key management."
    )]
    GenRoot {
        passphrase: Option<String>,
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
        #[structopt(
            long,
            short,
            help = "Derive device seed from root seed with this index"
        )]
        device_derivation_index: u64,
    },

    #[structopt(
        name = "genrevoke",
        about = "Generate a revocation seed given an encrypted root seed mnemonic, passphrase and derivation index."
    )]
    GenRevoke {
        #[structopt(
            long,
            short,
            help = "Derive revocation seed from root seed with this index"
        )]
        derivation_index: u64,
    },

    #[structopt(
        name = "revoke",
        about = "Produce the signed string needed to revoke a key given a revocation seed mnemonic and passphrase."
    )]
    Revoke {
        key: String,
    },
}

impl Dpki {
    pub fn execute(self) -> HcResult<String> {
        match self {
            Self::GenRoot{ passphrase } => genroot(passphrase),
            Self::Keygen{ path, keystore_passphrase, nullpass, quiet, root_seed, mnemonic_passphrase, device_derivation_index } =>
                keygen(path, keystore_passphrase, nullpass, mnemonic_passphrase, root_seed, Some(device_derivation_index), quiet)
                .map(|_| "success".to_string()),
            Self::GenRevoke{ derivation_index } => genrevoke(None, derivation_index),
            Self::Revoke { key } => revoke(None, key),
        }
    }
}

pub (crate) fn genroot(passphrase: Option<String>) -> HcResult<String> {
    let seed_buf = generate_random_seed_buf();
    let mut root_seed = RootSeed::new(seed_buf);
    match passphrase {
        Some(passphrase) => {
            root_seed.encrypt(passphrase, None)?.get_mnemonic()
        },
        None => {
            root_seed.seed_mut().get_mnemonic()
        }
    }
}

fn genrevoke(passphrase: Option<String>, derivation_index: u64) -> HcResult<String> {
    let root_seed_mnemonic = get_secure_string_double_check("Root Seed", false)?;
    let mut root_seed = match util::get_seed(root_seed_mnemonic, passphrase.clone(), SeedType::Root)? { TypedSeed::Root(s) => s, _ => unreachable!() };
    let mut revocation_seed = root_seed.generate_revocation_seed(derivation_index)?;
    match passphrase {
        Some(passphrase) => {
            revocation_seed.encrypt(passphrase, None)?.get_mnemonic()
        },
        None => {
            revocation_seed.seed_mut().get_mnemonic()
        }
    }
}

fn revoke(_passphrase: Option<String>, key_string: String) -> HcResult<String> {
    let revocation_seed_mnemonic = get_secure_string_double_check("Revocation Seed", false).expect("Could not obtain revocation seed");
    let _passphrase = _passphrase.unwrap_or_else(|| {
        get_secure_string_double_check("Revocation Seed Encryption Passphrase (placeholder)", false)
            .expect("Could not obtain passphrase")
    });

    let revocation_seed = Seed::new_with_mnemonic(revocation_seed_mnemonic, SeedType::Revocation)?;
    let mut revocation_seed = match revocation_seed.into_typed()? {
        TypedSeed::Revocation(inner_root_seed) => inner_root_seed,
        _ => unreachable!(),
    };

    let revocation_keypair = revocation_seed.generate_revocation_key(1)?;
    sign_with_key_from_seed(revocation_keypair, key_string)
}

fn sign_with_key_from_seed(mut keypair: KeyBundle, key_string: String) -> HcResult<String> {
    let mut data_buf = SecBuf::with_insecure_from_string(key_string);
    let mut signature_buf = keypair.sign(&mut data_buf)?;
    let buf = signature_buf.read_lock();
    let signature_str = base64::encode(&**buf);
    Ok(signature_str)
}
