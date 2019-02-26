use crate::{
    key_blob::*,
    key_bundle::{KeyBundle, SeedType},
    password_encryption::*,
    SEED_SIZE,
};
use bip39::{Language, Mnemonic};
use holochain_core_types::error::{HcResult, HolochainError};
use holochain_sodium::{kdf, pwhash, secbuf::SecBuf};
use std::str;

//--------------------------------------------------------------------------------------------------
// SeedInitializer
//--------------------------------------------------------------------------------------------------

/// Enum of all possible ways to initialize a Seed
pub enum SeedInitializer {
    Seed(SecBuf),
    Mnemonic(String),
}

//--------------------------------------------------------------------------------------------------
// Seed Types
//--------------------------------------------------------------------------------------------------

/// Enum of all the types of seeds
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SeedType {
    /// Root / Master seed
    Root,
    /// Revocation seed
    Revocation,
    /// Device specific seed
    Device,
    /// PIN Key for a device
    DevicePin,
    /// Application specific seed
    Application,
    /// Seed for a one use only key
    OneShot,
    /// Seed used only in tests or mocks
    Mock,
}

/// Enum of all the different behaviors a Seed can have
pub enum TypedSeed {
    Root(RootSeed),
    Device(DeviceSeed),
    DevicePin(DevicePinSeed),
}

/// Common Trait for TypedSeeds
pub trait SeedTrait {
    fn seed(&self) -> &Seed;
    fn seed_mut(&mut self) -> &mut Seed;
}

//--------------------------------------------------------------------------------------------------
// Seed
//--------------------------------------------------------------------------------------------------

// Data of a seed
#[derive(Debug)]
pub struct Seed {
    pub seed_type: SeedType,
    pub seed_buf: SecBuf,
}

impl Seed {
    pub fn new(seed_buf: SecBuf, seed_type: SeedType) -> Self {
        Seed {
            seed_type,
            seed_buf,
        }
    }

    // TODO: We need some way of zeroing the internal memory used by mnemonic
    pub fn new_with_mnemonic(phrase: String, seed_type: SeedType) -> HcResult<Self> {
        let maybe_mnemonic = Mnemonic::from_phrase(phrase, Language::English);
        if let Err(e) = maybe_mnemonic {
            return Err(HolochainError::Generic(&format!(
                "Error loading Mnemonic phrase: {}",
                e
            )));
        }
        let entropy: &[u8] = maybe_mnemonic.unwrap().entropy();
        assert_eq!(entropy.len(), SEED_SIZE);
        let mut seed_buf = SecBuf::with_secure(entropy.len());
        seed_buf.from_array(entropy);
        // Done
        Ok(Seed {
            seed_type,
            seed_buf,
        })
    }

    ///  Construct this seed struct from a SeedInitializer
    ///  @param {string} seed_type -
    ///  @param {SecBuf|string} initializer - data (buffer or mnemonic) for constructing the Seed
    pub fn new_with_initializer(initializer: SeedInitializer, seed_type: SeedType) -> Self {
        match initializer {
            SeedInitializer::Seed(seed_buf) => Seed::new(seed_buf, seed_type),
            SeedInitializer::Mnemonic(phrase) => Seed::new_with_mnemonic(phrase, SeedType)
                .expect("Invalid Mnemonic Seed initializer"),
        }
    }

    /// Generate a mnemonic for the seed.
    // TODO: We need some way of zeroing the internal memory used by mnemonic
    pub fn get_mnemonic(&mut self) -> HcResult<String> {
        let entropy = self.seed_buf.read_lock();
        let e = &*entropy;
        let mnemonic = Mnemonic::from_entropy(e, Language::English)?;
        Ok(mnemonic.phrase().to_string())
    }
}

//--------------------------------------------------------------------------------------------------
// RootSeed
//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct RootSeed {
    inner: Seed,
}

impl SeedTrait for RootSeed {
    fn seed(&self) -> &Seed {
        &self.inner
    }
    fn seed_mut(&mut self) -> &mut Seed {
        &mut self.inner
    }
}

impl RootSeed {
    /// Construct from a 32 bytes seed buffer
    pub fn new(seed_buf: SecBuf) -> Self {
        RootSeed {
            inner: Seed::new_with_initializer(SeedInitializer::Seed(seed_buf), SeedType::Root),
        }
    }

    /// Generate Device Seed
    /// @param {number} index - device index, must not be zero
    pub fn generate_device_seed(&mut self, index: u64) -> HcResult<DeviceSeed> {
        if index == 0 {
            return Err(HolochainError::ErrorGeneric("Invalid index".to_string()));
        }
        let mut device_seed_buf = SecBuf::with_secure(SEED_SIZE);
        let mut context = SecBuf::with_insecure_from_string("HCDEVICE".to_string());
        kdf::derive(
            &mut device_seed_buf,
            index,
            &mut context,
            &mut self.inner.seed_buf,
        )?;
        Ok(DeviceSeed::new(device_seed_buf))
    }
}

//--------------------------------------------------------------------------------------------------
// DeviceSeed
//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct DeviceSeed {
    inner: Seed,
}

impl SeedTrait for DeviceSeed {
    fn seed(&self) -> &Seed {
        &self.inner
    }
    fn seed_mut(&mut self) -> &mut Seed {
        &mut self.inner
    }
}

impl DeviceSeed {
    /// Construct from a 32 bytes seed buffer
    pub fn new(seed_buf: SecBuf) -> Self {
        DeviceSeed {
            inner: Seed::new_with_initializer(SeedInitializer::Seed(seed_buf), SeedType::Device),
        }
    }

    /// generate a device pin seed by applying pwhash of pin with this seed as the salt
    /// @param {string} pin - should be >= 4 characters 1-9
    /// @return {DevicePinSeed} Resulting Device Pin Seed
    pub fn generate_device_pin_seed(
        &mut self,
        pin: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<DevicePinSeed> {
        let mut hash = SecBuf::with_insecure(pwhash::HASHBYTES);
        util::pw_hash(pin, &mut self.inner.seed_buf, &mut hash, config)?;
        Ok(DevicePinSeed::new(hash))
    }
}

//--------------------------------------------------------------------------------------------------
// DevicePinSeed
//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct DevicePinSeed {
    inner: Seed,
}

impl SeedTrait for DevicePinSeed {
    fn seed(&self) -> &Seed {
        &self.inner
    }
    fn seed_mut(&mut self) -> &mut Seed {
        &mut self.inner
    }
}

impl DevicePinSeed {
    /// Construct from a 32 bytes seed buffer
    pub fn new(seed_buf: SecBuf) -> Self {
        DevicePinSeed {
            inner: Seed::new_with_initializer(SeedInitializer::Seed(seed_buf), SeedType::DevicePin),
        }
    }

    /// generate an application KeyBundle given an index based on this seed
    /// @param {number} index - device index, must not be zero
    /// @return {KeyBundle} Resulting keybundle
    pub fn generate_application_key(&mut self, index: u64) -> HcResult<KeyBundle> {
        if index == 0 {
            return Err(HolochainError::ErrorGeneric("Invalid index".to_string()));
        }
        let mut app_seed_buf = SecBuf::with_secure(SEED_SIZE);
        let mut context = SecBuf::with_insecure_from_string("HCAPPLIC".to_string());
        kdf::derive(
            &mut app_seed_buf,
            index,
            &mut context,
            &mut self.inner.seed_buf,
        )?;

        Ok(KeyBundle::new_from_seed_buf(
            &mut app_seed_buf,
            SeedType::Application,
        )?)
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use holochain_sodium::{pwhash, random::random_secbuf};
    use password_encryption::tests::TEST_CONFIG;

    fn test_generate_random_seed(s: usize) -> SecBuf {
        let mut seed_buf = SecBuf::with_insecure(s);
        seed_buf.randomize();
        seed_buf
    }

    #[test]
    fn it_should_create_a_new_seed() {
        let mut seed_buf = test_generate_random_seed(16);
        let seed_type = SeedType::OneShot;
        let seed = Seed::new_with_initializer(SeedInitializer::Seed(seed_buf), seed_type);
        assert_eq!(seed_type, seed.seed_type);
    }

    #[test]
    fn it_should_creat_a_new_root_seed() {
        let mut seed_buf = test_generate_random_seed(16);
        let root_seed = RootSeed::new(seed_buf);
        assert_eq!(SeedType::Root, root_seed.seed().seed_type);
    }

    #[test]
    fn it_should_create_a_device_seed() {
        let mut seed_buf = test_generate_random_seed(16);
        let mut root_seed = RootSeed::new(seed_buf_in);

        let mut device_seed_3 = root_seed.generate_device_seed(3).unwrap();
        assert_eq!(SeedType::Device, device_seed_3.seed().seed_type);
        let device_seed_err = root_seed.generate_device_seed(0).unwrap_err();
        assert_eq!(
            HolochainError::ErrorGeneric("Invalid index".to_string()),
            device_seed_err,
        );
        let mut device_seed_1 = root_seed.generate_device_seed(1).unwrap();
        let mut device_seed_3_b = root_seed.generate_device_seed(3).unwrap();
        assert!(device_seed_3.compare(&mut device_seed_3_b) == 0);
        assert!(device_seed_3.compare(&mut device_seed_1) != 0);
    }

    #[test]
    fn it_should_create_a_device_pin_seed() {
        let mut seed_buf = test_generate_random_seed(16);
        let mut pin = test_generate_random_seed(16);

        let mut root_seed = RootSeed::new(seed_buf);
        let mut device_seed = root_seed.generate_device_seed(3).unwrap();
        let device_pin_seed = device_seed
            .generate_device_pin_seed(&mut pin, TEST_CONFIG)
            .unwrap();
        assert_eq!(SeedType::DevicePin, device_pin_seed.seed().seed_type);
    }

    #[test]
    fn it_should_create_app_key_from_root_seed() {
        let mut seed_buf = test_generate_random_seed(16);
        let mut pin = test_generate_random_seed(16);

        let mut rs = RootSeed::new(seed_buf);
        let mut ds = rs.generate_device_seed(3).unwrap();
        let mut dps = ds.generate_device_pin_seed(&mut pin, TEST_CONFIG).unwrap();
        let mut keybundle_5 = dps.generate_application_key(5).unwrap();

        assert_eq!(crate::SIGNATURE_SIZE, keybundle_5.sign_priv.len());
        assert_eq!(SEED_SIZE, keybundle_5.enc_priv.len());
        assert_eq!(SeedType::Application, keybundle_5.seed_Type);

        let res = dps.generate_application_key(0);
        assert!(res.is_err());

        let mut keybundle_1 = dps.generate_application_key(1).unwrap();
        let mut keybundle_5_b = dps.generate_application_key(5).unwrap();
        assert!(keybundle_5.is_same(&mut keybundle_5_b));
        assert!(!keybundle_5.is_same(&mut keybundle_1));
    }

    #[test]
    fn it_should_roundtrip_mnemonic() {
        let mut seed_buf = SecBuf::with_insecure(16);
        {
            let mut seed_buf = seed_buf.write_lock();
            seed_buf[0] = 12;
            seed_buf[1] = 70;
            seed_buf[2] = 88;
        }
        let mut seed = Seed::new(seed_buf, SeedType::Root);
        let mnemonic = seed.get_mnemonic().unwrap();
        println!("mnemonic: {:?}", mnemonic);

        let mut seed_2 = Seed::new_with_mnemonic(mnemonic, seed_type).unwrap();
        assert_eq!(seed.seed_type, seed_2.seed_type);
        assert_eq!(0, seed.seed_mut().seed_buf.compare(&mut seed_2),);
    }
}
