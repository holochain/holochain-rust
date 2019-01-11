// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct KeyBundle {
    pub bundle_type: String,
    pub hint: String,
    pub data: String,
}
#[derive(RustcDecodable, RustcEncodable)]
pub struct Keys {
    pub pw_sign_pub: ReturnBundleData,
    pub pw_enc_pub: ReturnBundleData,
    pub pw_sign_priv: ReturnBundleData,
    pub pw_enc_priv: ReturnBundleData,
}
#[derive(RustcDecodable, RustcEncodable)]
pub struct ReturnBundleData {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub cipher: Vec<u8>,
}

/*
/**
 * This root seed should be pure entropy
 */
class RootSeed extends Seed {
/**
   * Get a new, completely random root seed
   */
  static async newRandom () {
    const seed = new mosodium.SecBuf(32)
    seed.randomize()
    return new RootSeed(seed)
  }

  /**
   * delegate to base class
   */
  async init (seed) {
    await super.init('hcRootSeed', seed)
  }

  /**
   * generate a device seed given an index based on this seed
   * @param {number} index
   * @return {DeviceSeed}
   */
  async getDeviceSeed (index) {
    if (typeof index !== 'number' || parseInt(index, 10) !== index || index < 1) {
      throw new Error('invalid index')
    }

    const seed = mosodium.kdf.derive(
      index, Buffer.from('HCDEVICE'), this._seed, this._seed.lockLevel())

    return new DeviceSeed(seed)
  }
}
*/
