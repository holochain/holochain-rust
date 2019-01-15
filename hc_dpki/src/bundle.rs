/// This struct is the bundle for the Key pairs. i.e. signing and encryption keys
///
/// The bundle_type tells if the bundle is a RootSeed bundle | DeviceSeed bundle | DevicePINSeed Bundle | ApplicationKeys Bundle
///
/// the data includes a base64 encoded string of the Keys Struct
pub struct KeyBundle {
    pub bundle_type: String,
    pub hint: String,
    pub data: String,
}

/// This struct is gives the sign and encryption key pairs
#[derive(RustcDecodable, RustcEncodable)]
pub struct Keys {
    pub pw_sign_pub: ReturnBundleData,
    pub pw_enc_pub: ReturnBundleData,
    pub pw_sign_priv: ReturnBundleData,
    pub pw_enc_priv: ReturnBundleData,
}

/// This struct type is for the return type for  util::pw_enc
#[derive(RustcDecodable, RustcEncodable)]
pub struct ReturnBundleData {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub cipher: Vec<u8>,
}
