use holochain_sodium::{
    pwhash::{
        OPSLIMIT_SENSITIVE,
        MEMLIMIT_SENSITIVE,
        ALG_ARGON3ID13,
        hash,
    },
    aead::{
        ABYTES,
        dec,
        enc,
    },
};
// allow overrides for unit-testing purposes
pub const pw_hash_ops_limit :u64 = OPSLIMIT_SENSITIVE;
pub const pw_hash_mem_limit :usize = MEMLIMIT_SENSITIVE;
pub const pw_hash_algo :i8 = ALG_ARGON3ID13;

/**
 * simplify the api for generating a password hash with our set parameters
 * @param {SecBuf} pass - the password buffer to hash
 * @param {Buffer} [salt] - if specified, hash with this salt (otherwise random)
 * @return {object} - { salt: Buffer, hash: SecBuf }
 */
pub fn pw_hash(password: &mut SecBuf,salt: Option<&mut SecBuf>,hash:&mut SecBuf){
    hash(&mut password,pw_hash_ops_limit,pw_hash_mem_limit,pw_hash_algo,salt,&mut hash).unwrap()
}

/**
 * Helper for encrypting a buffer with a pwhash-ed passphrase
 * @param {Buffer} data
 * @param {string} passphrase
 * @return {Buffer} - the encrypted data
 */
pub fn pw_enc(message:&mut SecBuf,passphrase:&mut SecBuf,salt:Option<&mut SecBuf>,nonce:&mut SecBuf,cipher:&mut SecBuf){
    let mut secret = SecBuf::with_secure(32);
    pw_hash(&mut passphrase,salt,&mut secret);
    enc(&mut message,&mut secret,None,&mut nonce,&mut cipher).unwrap()
}

/**
 * Helper for decrypting a buffer with a pwhash-ed passphrase
 * @param {Buffer} data
 * @param {string} passphrase
 * @return {Buffer} - the decrypted data
 */
pub fn pw_dec (passphrase:&mut SecBuf,salt:Option<&mut SecBuf>,nonce:&mut SecBuf,cipher:&mut SecBuf,decrypted_message:&mut SecBuf){
    let mut secret = SecBuf::with_secure(32);
    pw_hash(&mut passphrase,salt,&mut secret);
    dec(&mut decrypted_message,&mut secret,None,&mut nonce,&mut cipher).unwrap();
}
