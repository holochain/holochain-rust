use std::path::PathBuf;
use crate::util::{self, get_secure_string_double_check, user_prompt, user_prompt_yes_no, WordCountable};
use crate::cli::keygen;
use holochain_core_types::error::HcResult;
use holochain_dpki::{
    key_bundle::KeyBundle,
    seed::{RootSeed, SeedTrait, TypedSeed, SeedType, MnemonicableSeed},
    utils::generate_random_seed_buf,
};
use structopt::StructOpt;
use lib3h_sodium::secbuf::SecBuf;

const MNEMONIC_WORD_COUNT: usize = 24;
const ENCRYPTED_MNEMONIC_WORD_COUNT: usize = 2*MNEMONIC_WORD_COUNT;


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

        #[structopt(
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
            help = "Derive revocation seed from root seed with this index"
        )]
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
        name = "revoke",
        about = "Produce the signed string needed to revoke a key given a revocation seed mnemonic and passphrase."
    )]
    Revoke {
        #[structopt(
            help = "Public key to revoke (or any other string you want to sign with a revocation key)"
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
    },
}

impl Dpki {
    pub fn execute(self) -> HcResult<String> {
        match self {
            Self::GenRoot{ passphrase, quiet } => genroot(passphrase, quiet),
            Self::Keygen{ path, keystore_passphrase, nullpass, quiet, root_seed, mnemonic_passphrase, device_derivation_index } =>
                keygen(path, keystore_passphrase, nullpass, mnemonic_passphrase, root_seed, Some(device_derivation_index), quiet)
                .map(|_| "success".to_string()),
            Self::GenRevoke{ derivation_index, root_seed_passphrase, revocation_seed_passphrase, quiet } => genrevoke(root_seed_passphrase, revocation_seed_passphrase, derivation_index, quiet),
            Self::Revoke { passphrase, key, quiet } => revoke(passphrase, key, quiet),
        }
    }
}

fn genroot(passphrase: Option<String>, quiet: bool) -> HcResult<String> {
    user_prompt("This will generate a new random DPKI root seed.
You should only have to do this once and you should keep the seed safe.
It will be printed out once as a mnemonic at the end of this process.
The root seed can be used to generate new device, revocation and auth keys.\n", quiet);

    let passphrase = passphrase.or_else(|| {
        match user_prompt_yes_no("Would you like to encrypt the root seed?", quiet) {
            true => Some(get_secure_string_double_check("Root Seed Passphrase", quiet).expect("Could not read revocation passphrase")),
            false => None,
        }
    });
    println!();
    genroot_inner(passphrase)
}

pub (crate) fn genroot_inner(passphrase: Option<String>) -> HcResult<String> {
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

fn genrevoke(root_seed_passphrase: Option<String>, revocation_seed_passphrase: Option<String>, derivation_index: u64, quiet: bool) -> HcResult<String> {
    user_prompt("This will generate a new revocation seed derived from a root seed.
This can be used to revoke access to keys you have previously authorized.\n", quiet);


    let root_seed_mnemonic = get_secure_string_double_check("Root Seed", quiet)?;
    let root_seed_passphrase = match root_seed_mnemonic.word_count() {
        MNEMONIC_WORD_COUNT => None, // ignore any passphrase passed if it is an unencrypted mnemonic
        ENCRYPTED_MNEMONIC_WORD_COUNT => root_seed_passphrase.or_else(|| { Some(get_secure_string_double_check("Root Seed Passphrase", quiet).expect("Could not read passphrase")) }),
        _ => panic!("Invalid word count for mnemonic")
    };
    let revocation_seed_passphrase = revocation_seed_passphrase.or_else(|| {
        match user_prompt_yes_no("Would you like to encrypt the revocation seed?", quiet) {
            true => Some(get_secure_string_double_check("Revocation Seed Passphrase", quiet).expect("Could not read revocation passphrase")),
            false => None,
        }
    });
    println!();
    genrevoke_inner(root_seed_mnemonic, root_seed_passphrase, revocation_seed_passphrase, derivation_index)
}

fn genrevoke_inner(root_seed_mnemonic: String, root_seed_passphrase: Option<String>, revocation_seed_passphrase: Option<String>, derivation_index: u64) -> HcResult<String> {
    let mut root_seed = match util::get_seed(root_seed_mnemonic, root_seed_passphrase, SeedType::Root)? { TypedSeed::Root(s) => s, _ => unreachable!() };
    let mut revocation_seed = root_seed.generate_revocation_seed(derivation_index)?;
    match revocation_seed_passphrase {
        Some(passphrase) => {
            revocation_seed.encrypt(passphrase, None)?.get_mnemonic()
        },
        None => {
            revocation_seed.seed_mut().get_mnemonic()
        }
    }
}

fn revoke(passphrase: Option<String>, key_string: String, quiet: bool) -> HcResult<String> {
    user_prompt("This will sign a given key/string with a revocation key.
The resulting signed message can be used to publish a DPKI revocation message which will revoke the key.\n", quiet);

    let revocation_seed_mnemonic = get_secure_string_double_check("Revocation Seed", false)?;
    let passphrase = match revocation_seed_mnemonic.word_count() {
        MNEMONIC_WORD_COUNT => None, // ignore any passphrase passed if it is an unencrypted mnemonic
        ENCRYPTED_MNEMONIC_WORD_COUNT => passphrase.or_else(|| { Some(get_secure_string_double_check("Revocation Seed Passphrase", quiet).expect("Could not read passphrase")) }),
        _ => panic!("Invalid word count for mnemonic")
    };
    println!();
    revoke_inner(revocation_seed_mnemonic, passphrase, key_string)
}

fn revoke_inner(revocation_seed_mnemonic: String, passphrase: Option<String>, key_string: String) -> HcResult<String> {
    let mut revocation_seed = match util::get_seed(revocation_seed_mnemonic, passphrase, SeedType::Revocation)? { TypedSeed::Revocation(s) => s, _ => unreachable!() };
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
