use std::path::PathBuf;
use crate::util::get_secure_string_double_check;
use crate::cli::keygen;
use holochain_core_types::error::HcResult;
use holochain_dpki::{
    seed::{RootSeed, SeedTrait},
    utils::generate_random_seed_buf,
};
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Dpki {
    #[structopt(name = "genroot")]
    GenRoot {
        passphrase: Option<String>,
    },
    #[structopt(name = "keygen")]
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
        passphrase: Option<String>,
        #[structopt(
            long,
            short,
            help = "Set root seed via argument and don't prompt for it (not reccomended). BIP39 mnemonic encoded root seed to derive device seed and agent key from"
        )]
        root_seed: Option<String>,
        #[structopt(
            long,
            short,
            help = "Derive device seed from root seed with this index"
        )]
        device_derivation_index: u64,
    },
    #[structopt(name = "genrevoke")]
    GenRevoke {
        #[structopt(
            long,
            short,
            help = "Derive derivation seed from root seed with this index"
        )]
        derivation_index: u64,
    },
    #[structopt(name = "revoke")]
    Revoke {
        key: String,
    },
}

impl Dpki {
    pub fn execute(self) -> HcResult<String> {
        match self {
            Self::GenRoot{ passphrase } => genroot(passphrase),
            Self::Keygen{ path, passphrase, quiet, root_seed, device_derivation_index } =>
                keygen(path, passphrase, quiet, root_seed, Some(device_derivation_index))
                .map(|_| "success".to_string()),
            Self::GenRevoke{ derivation_index } => genrevoke(None, derivation_index),
            Self::Revoke { .. } => {
                panic!()
            },
        }
    }
}

pub (crate) fn genroot(_passphrase: Option<String>) -> HcResult<String> {
    let seed_buf = generate_random_seed_buf();
    let mut root_seed = RootSeed::new(seed_buf);

    // prompt for a passphrase to encrypt the root seed.
    // TODO: Actually encrypt to root seed. Passphrase is not used at this time
    let _passphrase = _passphrase.unwrap_or_else(|| {
        get_secure_string_double_check("Root Key Encryption Passphrase (placeholder)", false)
            .expect("Could not obtain passphrase")
    });

    root_seed.seed_mut().get_mnemonic()
}

fn genrevoke(_passphrase: Option<String>, derivation_index: u64) -> HcResult<String> {
    let mut root_seed = keygen::get_root_seed(None, &String::from(""), false)?;
    let mut revocation_seed = root_seed.generate_revocation_seed(derivation_index)?;

    // prompt for a passphrase to encrypt the root seed.
    // TODO: Actually encrypt to root seed. Passphrase is not used at this time
    let _passphrase = _passphrase.unwrap_or_else(|| {
        get_secure_string_double_check("Revocation Key Encryption Passphrase (placeholder)", false)
            .expect("Could not obtain passphrase")
    });

    revocation_seed.seed_mut().get_mnemonic()
}
