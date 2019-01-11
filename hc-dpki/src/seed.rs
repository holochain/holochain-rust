use super::seed::InitializeSeed::{MnemonicInit, SeedInit};
use crate::{
    bundle,
    holochain_sodium::{kdf, pwhash, random::random_secbuf, secbuf::SecBuf},
    keypair::Keypair,
    util,
};
use rustc_serialize::json;

pub enum InitializeSeed {
    SeedInit(SecBuf),
    MnemonicInit(String),
}

#[derive(Debug)]
pub struct Seed {
    seed_type: String,
    seed_buf: SecBuf,
}

impl Seed {
    /**
     * generate a persistence bundle with hint info
     * @param {string} passphrase - the encryption passphrase
     * @param {string} hint - additional info / description for persistence
     */
    pub fn get_seed_bundle(&mut self, passphrase: String, hint: String) -> bundle::KeyBundle {
        let mut passphrase = SecBuf::with_insecure_from_string(passphrase);
        let seed_data: bundle::ReturnBundleData = util::pw_enc(&mut self.seed_buf, &mut passphrase);

        // convert -> to string -> to base64
        let seed_data_serialized = json::encode(&seed_data).unwrap();
        let seed_data_encoded = base64::encode(&seed_data_serialized);

        bundle::KeyBundle {
            bundle_type: self.seed_type.clone(),
            hint,
            data: seed_data_encoded,
        }
    }

    pub fn new(stype: &String, sm: InitializeSeed) -> Self {
        match sm {
            SeedInit(s) => Seed {
                seed_type: stype.clone(),
                seed_buf: s,
            },
            MnemonicInit(m) => Seed {
                seed_type: stype.clone(),
                // TODO : incorect implementation
                seed_buf: SecBuf::with_insecure_from_string(m),
            },
        }
    }

    // pub fn from_bundle(bundle: bundle::KeyBundle,passphrase: String){
    //     match bundle.bundle_type {
    //         "hcDeviceSeed" => expr,
    //         "hcDevicePinSeed" => expr,
    //         "hcRootSeed" => expr,
    //         _ => panic!("Invalid Bundle Type!"),
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holochain_sodium::random::random_secbuf;

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

        // println!("SEED: {:?}",s.seed_type);
        assert_eq!("hcRootSeed".to_string(), rs.s.seed_type);
    }

    #[test]
    fn it_should_create_a_random_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let rs = RootSeed::new_random();

        // println!("SEED: {:?}",s.seed_type);
        assert_eq!("hcRootSeed".to_string(), rs.s.seed_type);
    }
    #[test]
    fn creating_seed_bundle() {
        // Initialized a Seed
        let mut seed_buf_in = SecBuf::with_insecure(32);
        random_secbuf(&mut seed_buf_in);
        let mut s = Seed {
            seed_type: "hcRootSeed".to_string(),
            seed_buf: seed_buf_in,
        };

        let b: bundle::KeyBundle =
            s.get_seed_bundle("PASSWORD!LNFA*".to_string(), "hint".to_string());

        println!("Bundle type:{:?}", b.bundle_type);
        println!("Bundle data:{:?}", b.data);

        assert_eq!("hcRootSeed".to_string(), b.bundle_type);
    }

    #[test]
    fn it_should_create_a_device_seed_from_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let mut rs = RootSeed::new_random();

        let ds: DeviceSeed = rs.get_device_seed(3);
        // println!("SEED: {:?}",s.seed_type);
        assert_eq!("hcDeviceSeed".to_string(), ds.s.seed_type);
    }

    #[test]
    fn it_should_create_a_device_pin_seed_from_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let mut rs = RootSeed::new_random();

        let mut ds: DeviceSeed = rs.get_device_seed(3);
        // println!("SEED: {:?}",s.seed_type);

        let dps: DevicePinSeed = ds.get_device_pin_seed("1802".to_string());

        assert_eq!("hcDevicePinSeed".to_string(), dps.s.seed_type);
    }

    #[test]
    fn it_should_create_application_from_root_seed() {
        let mut seed_buf_in = SecBuf::with_insecure(16);
        random_secbuf(&mut seed_buf_in);

        let mut rs = RootSeed::new_random();

        let mut ds: DeviceSeed = rs.get_device_seed(3);
        // println!("SEED: {:?}",s.seed_type);

        let mut dps: DevicePinSeed = ds.get_device_pin_seed("1802".to_string());

        let keys = dps.get_application_keypair(5);

        assert_eq!(64, keys.sign_priv.len());
        assert_eq!(32, keys.enc_priv.len());
        // assert_eq!("hcDevicePinSeed".to_string(),keys.pub_keys);
    }
}

// #[warn(dead_code)]
pub struct DevicePinSeed {
    s: Seed,
}

impl DevicePinSeed {
    /**
     * delegate to base struct
     */
    pub fn new(s: SecBuf) -> Self {
        DevicePinSeed {
            s: Seed::new(&"hcDevicePinSeed".to_string(), SeedInit(s)),
        }
    }

    /**
     * generate an application keypair given an index based on this seed
     * @param {number} index
     * @return {Keypair}
     */
    pub fn get_application_keypair(&mut self, index: u64) -> Keypair {
        if index < 1 {
            panic!("invalid index");
        }

        let mut out_seed = SecBuf::with_insecure(32);
        let mut placeholder = SecBuf::with_insecure_from_string("HCAPPLIC".to_string());
        kdf::derive(
            &mut out_seed,
            index.clone(),
            &mut placeholder,
            &mut self.s.seed_buf,
        )
        .unwrap();

        Keypair::new_from_seed(&mut out_seed)
    }
}

pub struct DeviceSeed {
    s: Seed,
}

impl DeviceSeed {
    /**
     * delegate to base struct
     */
    pub fn new(s: SecBuf) -> Self {
        DeviceSeed {
            s: Seed::new(&"hcDeviceSeed".to_string(), SeedInit(s)),
        }
    }

    /**
     * generate a device pin seed by applying pwhash of pin with this seed as the salt
     * @param {string} pin - should be >= 4 characters 1-9
     * @return {DevicePinSeed}
     */
    pub fn get_device_pin_seed(&mut self, pin: String) -> DevicePinSeed {
        if pin.len() < 4 {
            panic!("invalid PIN Size");
        }
        // let pin_encoded = base64::encode(&pin);
        let mut pin_buf = SecBuf::with_insecure_from_string(pin);

        let mut hash = SecBuf::with_insecure(pwhash::HASHBYTES);

        util::pw_hash(&mut pin_buf, &mut self.s.seed_buf, &mut hash);

        DevicePinSeed::new(hash)
    }
}

#[derive(Debug)]
pub struct RootSeed {
    s: Seed,
}

impl RootSeed {
    /**
     * Get a new, completely random root seed
     */
    pub fn new_random() -> Self {
        let mut s = SecBuf::with_insecure(32);
        random_secbuf(&mut s);
        RootSeed::new(s)
    }

    /**
     * delegate to base struct
     */
    pub fn new(s: SecBuf) -> Self {
        RootSeed {
            s: Seed::new(&"hcRootSeed".to_string(), SeedInit(s)),
        }
    }

    pub fn get_device_seed(&mut self, index: u64) -> DeviceSeed {
        if index < 1 {
            panic!("invalid index");
        }
        let mut out_seed = SecBuf::with_insecure(32);
        let mut placeholder = SecBuf::with_insecure_from_string("HCDEVICE".to_string());
        kdf::derive(
            &mut out_seed,
            index.clone(),
            &mut placeholder,
            &mut self.s.seed_buf,
        )
        .unwrap();
        DeviceSeed::new(out_seed)
    }
}
