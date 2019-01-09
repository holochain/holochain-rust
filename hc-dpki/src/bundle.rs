pub struct KeyBundle {
    pub bundle_type: String,
    pub hint: String,
    pub data: Keys,
}
pub struct Keys {
    pub pw_pub_keys: ReturnBundleData,
    pub pw_sign_priv: ReturnBundleData,
    pub pw_enc_priv: ReturnBundleData,
}
pub struct ReturnBundleData {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub cipher: Vec<u8>,
}