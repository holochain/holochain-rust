use super::seed::InitializeSeed::{MnemonicInit, SeedInit};
use crate::{
    bundle,
    holochain_sodium::{kdf, pwhash, secbuf::SecBuf},
    keypair::Keypair,
    util::{self, PwHashConfig},
};
use bip39::{Language, Mnemonic};
use boolinator::*;
use holochain_core_types::error::HolochainError;
use rustc_serialize::json;
use std::str;

pub enum InitializeSeed {
    SeedInit(SecBuf),
    MnemonicInit(String),
}

#[derive(Debug)]
pub struct Seed {
    seed_type: String,
    seed_buf: SecBuf,
}

pub enum FromBundle {
    Rs(RootSeed),
    Ds(DeviceSeed),
    Dps(DevicePinSeed),
    Nill,
}

impl Seed {
    /// Get the proper seed type from a persistence bundle
    ///
    /// @param {object} bundle - the persistence bundle
    ///
    /// @param {string} passphrase - the decryption passphrase
    ///
    /// @return Type FromBundle
    pub fn from_seed_bundle(
        bundle: bundle::KeyBundle,
        passphrase: String,
        config: Option<PwHashConfig>,
    ) -> Result<FromBundle, HolochainError> {
        let mut passphrase = SecBuf::with_insecure_from_string(passphrase);

        let seed_data_decoded = base64::decode(&bundle.data)?;
        let seed_data_string = str::from_utf8(&seed_data_decoded)?;

        let seed_data_deserialized: bundle::ReturnBundleData =
            json::decode(&seed_data_string)?;
        let mut seed_data = SecBuf::with_secure(32);

        util::pw_dec(
            &seed_data_deserialized,
            &mut passphrase,
            &mut seed_data,
            config,
        )?;

        match bundle.bundle_type.as_ref() {
            "hcRootSeed" => Ok(FromBundle::Rs(RootSeed::new(seed_data))),
            "hcDeviceSeed" => Ok(FromBundle::Ds(DeviceSeed::new(seed_data))),
            "hcDevicePinSeed" => Ok(FromBundle::Dps(DevicePinSeed::new(seed_data))),
            _ => Err(HolochainError::new(&format!("Invalid Bundle type"))),
        }
    }

    ///  generate a persistence bundle with hint info
    ///
    ///  @param {string} passphrase - the encryption passphrase
    ///
    ///  @param {string} hint - additional info / description for persistence
    ///
    /// @return {KeyBundle} - bundle of the seed
    pub fn get_seed_bundle(
        &mut self,
        passphrase: String,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> Result<bundle::KeyBundle, HolochainError> {
        let mut passphrase = SecBuf::with_insecure_from_string(passphrase);
        let seed_data: bundle::ReturnBundleData =
            util::pw_enc(&mut self.seed_buf, &mut passphrase, config)?;

        // convert -> to string -> to base64
        let seed_data_serialized = json::encode(&seed_data)?;
        let seed_data_encoded = base64::encode(&seed_data_serialized);

        Ok(bundle::KeyBundle {
            bundle_type: self.seed_type.clone(),
            hint,
            data: seed_data_encoded,
        })
    }

    ///  Initialize this seed class with persistence bundle type and private seed
    ///
    ///  @param {string} stype - the persistence bundle type
    ///
    ///  @param {SecBuf|string} seed - the private seed data (as a buffer or mnemonic)
    pub fn new(stype: &String, sm: InitializeSeed) -> Self {
        match sm {
            SeedInit(s) => Seed {
                seed_type: stype.clone(),
                seed_buf: s,
            },
            MnemonicInit(phrase) => {
                let mnemonic = Mnemonic::from_phrase(phrase, Language::English).unwrap();
                let entropy: &[u8] = mnemonic.entropy();
                let mut buf = SecBuf::with_insecure(entropy.len());
                util::convert_array_to_secbuf(entropy, &mut buf);
                Seed {
                    seed_type: stype.clone(),
                    seed_buf: buf,
                }
            }
        }
    }

    /// Generated a mnemonic for the seed.
    pub fn get_mnemonic(&mut self) -> Result<String, HolochainError> {
        let entropy = self.seed_buf.read_lock();
        let e = &*entropy;
        let mnemonic = Mnemonic::from_entropy(e, Language::English).unwrap();
        Ok(mnemonic.phrase().to_string())
    }
}
#[derive(Debug)]
pub struct DevicePinSeed {
    s: Seed,
}

impl DevicePinSeed {
    /// creates bundle for for the seed
    pub fn get_bundle(
        &mut self,
        passphrase: String,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> Result<bundle::KeyBundle, HolochainError> {
        Ok(self.s.get_seed_bundle(passphrase, hint, config)?)
    }

    /// delegate to base struct
    pub fn new(s: SecBuf) -> Self {
        DevicePinSeed {
            s: Seed::new(&"hcDevicePinSeed".to_string(), SeedInit(s)),
        }
    }

    /// generate an application keypair given an index based on this seed
    /// @param {number} index
    /// @return {Keypair}
    pub fn get_application_keypair(&mut self, index: u64) -> Result<Keypair, HolochainError> {
        (index >= 1).ok_or(HolochainError::ErrorGeneric("Invalid index".to_string()))?;

        let mut out_seed = SecBuf::with_insecure(32);
        let mut placeholder = SecBuf::with_insecure_from_string("HCAPPLIC".to_string());
        kdf::derive(
            &mut out_seed,
            index.clone(),
            &mut placeholder,
            &mut self.s.seed_buf,
        )?;

        Ok(Keypair::new_from_seed(&mut out_seed)?)
    }
}
#[derive(Debug)]
pub struct DeviceSeed {
    s: Seed,
}

impl DeviceSeed {
    /// creates bundle for for the seed
    pub fn get_bundle(
        &mut self,
        passphrase: String,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> Result<bundle::KeyBundle, HolochainError> {
        Ok(self.s.get_seed_bundle(passphrase, hint, config)?)
    }

    /// delegate to base struct
    pub fn new(s: SecBuf) -> Self {
        DeviceSeed {
            s: Seed::new(&"hcDeviceSeed".to_string(), SeedInit(s)),
        }
    }

    /// generate a device pin seed by applying pwhash of pin with this seed as the salt
    /// @param {string} pin - should be >= 4 characters 1-9
    /// @return {DevicePinSeed}
    pub fn get_device_pin_seed(
        &mut self,
        pin: String,
        config: Option<PwHashConfig>,
    ) -> Result<DevicePinSeed, HolochainError> {
        (pin.len() >= 4).ok_or(HolochainError::ErrorGeneric("Invalid PIN Size".to_string()))?;

        // let pin_encoded = base64::encode(&pin);
        let mut pin_buf = SecBuf::with_insecure_from_string(pin);

        let mut hash = SecBuf::with_insecure(pwhash::HASHBYTES);

        util::pw_hash(&mut pin_buf, &mut self.s.seed_buf, &mut hash, config)?;

        Ok(DevicePinSeed::new(hash))
    }
}

#[derive(Debug)]
pub struct RootSeed {
    s: Seed,
}

impl RootSeed {
    /// creates bundle for for the seed
    pub fn get_bundle(
        &mut self,
        passphrase: String,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> Result<bundle::KeyBundle, HolochainError> {
        Ok(self.s.get_seed_bundle(passphrase, hint, config)?)
    }

    /// delegate to base struct
    pub fn new(s: SecBuf) -> Self {
        RootSeed {
            s: Seed::new(&"hcRootSeed".to_string(), SeedInit(s)),
        }
    }

    /// Generate Device Seed
    pub fn get_device_seed(&mut self, index: u64) -> Result<DeviceSeed, HolochainError> {
        (index >= 1).ok_or(HolochainError::ErrorGeneric("Invalid index".to_string()))?;

        let mut out_seed = SecBuf::with_insecure(32);
        let mut placeholder = SecBuf::with_insecure_from_string("HCDEVICE".to_string());
        kdf::derive(
            &mut out_seed,
            index.clone(),
            &mut placeholder,
            &mut self.s.seed_buf,
        )?;
        Ok(DeviceSeed::new(out_seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holochain_sodium::{pwhash, random::random_secbuf};

    const TEST_CONFIG: Option<PwHashConfig> = Some(PwHashConfig(
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE,
        pwhash::ALG_ARGON2ID13,
    ));

    #[test]
    fn it_should_creat_a_new_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let seed_type = "hcRootSeed".to_string();

        let s = Seed::new(&seed_type, SeedInit(seed_buf_in));

        println!("SEED: {:?}", s.seed_type);
        assert_eq!(seed_type, s.seed_type);
    }

    #[test]
    fn it_should_creat_a_new_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let rs = RootSeed::new(seed_buf_in);

        assert_eq!("hcRootSeed".to_string(), rs.s.seed_type);
    }

    #[test]
    fn creating_seed_bundle() {
        let mut seed_buf_in = SecBuf::with_insecure(32);
        random_secbuf(&mut seed_buf_in);
        let mut s = Seed {
            seed_type: "hcRootSeed".to_string(),
            seed_buf: seed_buf_in,
        };

        let b: bundle::KeyBundle = s
            .get_seed_bundle(
                "PASSWORD!LNFA*".to_string(),
                "hint".to_string(),
                TEST_CONFIG,
            )
            .unwrap();

        println!("Bundle type:{:?}", b.bundle_type);
        println!("Bundle data:{:?}", b.data);

        assert_eq!("hcRootSeed".to_string(), b.bundle_type);
    }

    #[test]
    fn it_should_create_0_device_seed_from_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let mut rs = RootSeed::new(seed_buf_in);

        let seed: HolochainError = rs.get_device_seed(0).unwrap_err();
        assert_eq!(
            HolochainError::ErrorGeneric("Invalid index".to_string()),
            seed
        );
    }

    #[test]
    fn it_should_create_a_device_seed_from_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let mut rs = RootSeed::new(seed_buf_in);

        let ds: DeviceSeed = rs.get_device_seed(3).unwrap();
        assert_eq!("hcDeviceSeed".to_string(), ds.s.seed_type);
    }
    #[test]
    fn it_should_error_with_invalid_device_pin_seed_from_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let mut rs = RootSeed::new(seed_buf_in);

        let mut ds: DeviceSeed = rs.get_device_seed(3).unwrap();
        let seed: HolochainError = ds
            .get_device_pin_seed("802".to_string(), TEST_CONFIG)
            .unwrap_err();
        assert_eq!(
            HolochainError::ErrorGeneric("Invalid PIN Size".to_string()),
            seed
        );
    }

    #[test]
    fn it_should_create_a_device_pin_seed_from_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let mut rs = RootSeed::new(seed_buf_in);

        let mut ds: DeviceSeed = rs.get_device_seed(3).unwrap();
        let dps: DevicePinSeed = ds
            .get_device_pin_seed("1802".to_string(), TEST_CONFIG)
            .unwrap();

        assert_eq!("hcDevicePinSeed".to_string(), dps.s.seed_type);
    }

    #[test]
    fn it_should_create_application_from_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let mut rs = RootSeed::new(seed_buf_in);

        let mut ds: DeviceSeed = rs.get_device_seed(3).unwrap();
        let mut dps: DevicePinSeed = ds
            .get_device_pin_seed("1802".to_string(), TEST_CONFIG)
            .unwrap();

        let keys = dps.get_application_keypair(5).unwrap();

        assert_eq!(64, keys.sign_priv.len());
        assert_eq!(32, keys.enc_priv.len());
    }

    #[test]
    fn creating_seed_bundle_and_getting_the_seed_back() {
        let mut seed_buf_in = SecBuf::with_insecure(32);
        random_secbuf(&mut seed_buf_in);
        let mut initial_seed = Seed {
            seed_buf: seed_buf_in,
            seed_type: "hcRootSeed".to_string(),
        };
        let passphrase: String = "PASSWORD!LNFA*".to_string();
        let b: bundle::KeyBundle = initial_seed
            .get_seed_bundle(
                "PASSWORD!LNFA*".to_string(),
                "hint".to_string(),
                TEST_CONFIG,
            )
            .unwrap();

        let s: FromBundle = Seed::from_seed_bundle(b, passphrase, TEST_CONFIG).unwrap();

        match s {
            FromBundle::Rs(mut rs) => {
                let fs = rs.s.seed_buf.read_lock();
                let is = initial_seed.seed_buf.read_lock();
                assert_eq!(format!("{:?}", *fs), format!("{:?}", *is));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn creating_root_seed_bundle_and_getting_the_seed_back() {
        let mut seed_buf_in = SecBuf::with_insecure(32);
        random_secbuf(&mut seed_buf_in);
        let mut initial_root_seed = RootSeed::new(seed_buf_in);
        let passphrase: String = "PASSWORD!LNFA*".to_string();
        let b: bundle::KeyBundle = initial_root_seed
            .get_bundle(
                "PASSWORD!LNFA*".to_string(),
                "hint".to_string(),
                TEST_CONFIG,
            )
            .unwrap();

        let s: FromBundle = Seed::from_seed_bundle(b, passphrase, TEST_CONFIG).unwrap();

        match s {
            FromBundle::Rs(mut rs) => {
                let fs = rs.s.seed_buf.read_lock();
                let is = initial_root_seed.s.seed_buf.read_lock();
                assert_eq!(format!("{:?}", *fs), format!("{:?}", *is));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn creating_device_seed_bundle_and_getting_the_seed_back() {
        let mut seed_buf_in = SecBuf::with_insecure(32);
        random_secbuf(&mut seed_buf_in);
        let mut initial_device_seed = DeviceSeed::new(seed_buf_in);
        let passphrase: String = "PASSWORD!LNFA*".to_string();
        let b: bundle::KeyBundle = initial_device_seed
            .get_bundle(
                "PASSWORD!LNFA*".to_string(),
                "hint".to_string(),
                TEST_CONFIG,
            )
            .unwrap();

        println!("TYPE: {}", b.bundle_type);
        let s: FromBundle = Seed::from_seed_bundle(b, passphrase, TEST_CONFIG).unwrap();

        match s {
            FromBundle::Ds(mut rs) => {
                let fs = rs.s.seed_buf.read_lock();
                let is = initial_device_seed.s.seed_buf.read_lock();
                println!("Seed {:?}", fs);
                println!("name {:?}", is);
                assert_eq!(format!("{:?}", *fs), format!("{:?}", *is));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn creating_device_pin_seed_bundle_and_getting_the_seed_back() {
        // Initialized a Seed
        let mut seed_buf_in = SecBuf::with_insecure(32);
        random_secbuf(&mut seed_buf_in);
        let mut initial_device_pin_seed = DevicePinSeed::new(seed_buf_in);
        let passphrase: String = "PASSWORD!LNFA*".to_string();
        let b: bundle::KeyBundle = initial_device_pin_seed
            .get_bundle(
                "PASSWORD!LNFA*".to_string(),
                "hint".to_string(),
                TEST_CONFIG,
            )
            .unwrap();

        println!("TYPE: {}", b.bundle_type);
        let s: FromBundle = Seed::from_seed_bundle(b, passphrase, TEST_CONFIG).unwrap();

        match s {
            FromBundle::Dps(mut rs) => {
                let fs = rs.s.seed_buf.read_lock();
                let is = initial_device_pin_seed.s.seed_buf.read_lock();
                println!("Seed {:?}", fs);
                println!("name {:?}", is);
                assert_eq!(format!("{:?}", *fs), format!("{:?}", *is));
            }
            _ => assert!(false),
        }
    }
    #[test]
    fn it_should_create_a_mnemonic_and_get_seed_back() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        {
            let mut seed_buf_in = seed_buf_in.write_lock();
            seed_buf_in[0] = 12;
            seed_buf_in[1] = 70;
            seed_buf_in[2] = 88;
        }
        let seed_type = "hcRootSeed".to_string();

        let mut s = Seed::new(&seed_type, SeedInit(seed_buf_in));

        let m = s.get_mnemonic().unwrap();
        println!("Menemenoc: {:?}", m);

        let seed_type = "hcRootSeed".to_string();

        let mut rs = Seed::new(&seed_type, MnemonicInit(m));

        println!("SEED: {:?}", s.seed_type);

        let fs = s.seed_buf.read_lock();
        let is = rs.seed_buf.read_lock();
        assert_eq!(format!("{:?}", *fs), format!("{:?}", *is));
    }

    #[test]
    fn it_should_create_a_mnemonic() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        {
            let mut seed_buf_in = seed_buf_in.write_lock();
            seed_buf_in[0] = 12;
            seed_buf_in[1] = 70;
            seed_buf_in[2] = 88;
        }
        let seed_type = "hcRootSeed".to_string();

        let mut s = Seed::new(&seed_type, SeedInit(seed_buf_in));

        let m = s.get_mnemonic().unwrap();
        println!("Menemenoc: {:?}", m);
        assert_eq!("arrange crazy abandon abandon abandon abandon abandon abandon abandon abandon abandon absent".to_string(), m);
    }

}
