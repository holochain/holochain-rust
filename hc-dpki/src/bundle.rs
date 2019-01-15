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
