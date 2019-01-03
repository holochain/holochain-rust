use holochain_sodium::{
    secbuf::{SecBuf,},
    pwhash::{
        OPSLIMIT_SENSITIVE,
        MEMLIMIT_SENSITIVE,
        ALG_ARGON2ID13,
        hash,
    },
    aead::{
        ABYTES,
        dec,
        enc,
    },
};
// allow overrides for unit-testing purposes
// pub const pw_hash_ops_limit :u64 = OPSLIMIT_SENSITIVE;
// pub const pw_hash_mem_limit :usize = MEMLIMIT_SENSITIVE;
// pub const pw_hash_algo :i8 = ALG_ARGON2ID13;

// /**
//  * simplify the api for generating a password hash with our set parameters
//  * @param {SecBuf} pass - the password buffer to hash
//  * @param {Buffer} [salt] - if specified, hash with this salt (otherwise random)
//  * @return {object} - { salt: Buffer, hash: SecBuf }
//  */
// pub fn pw_hash(password: &mut SecBuf,salt: Option<&mut SecBuf>,hash:&mut SecBuf){
//     hash(&mut password,pw_hash_ops_limit,pw_hash_mem_limit,pw_hash_algo,salt,&mut hash).unwrap()
// }
//
// /**
//  * Helper for encrypting a buffer with a pwhash-ed passphrase
//  * @param {Buffer} data
//  * @param {string} passphrase
//  * @return {Buffer} - the encrypted data
//  */
// pub fn pw_enc(message:&mut SecBuf,passphrase:&mut SecBuf,salt:Option<&mut SecBuf>,nonce:&mut SecBuf,cipher:&mut SecBuf){
//     let mut secret = SecBuf::with_secure(32);
//     pw_hash(&mut passphrase,salt,&mut secret);
//     enc(&mut message,&mut secret,None,&mut nonce,&mut cipher).unwrap()
// }
//
// /**
//  * Helper for decrypting a buffer with a pwhash-ed passphrase
//  * @param {Buffer} data
//  * @param {string} passphrase
//  * @return {Buffer} - the decrypted data
//  */
// pub fn pw_dec (passphrase:&mut SecBuf,salt:Option<&mut SecBuf>,nonce:&mut SecBuf,cipher:&mut SecBuf,decrypted_message:&mut SecBuf){
//     let mut secret = SecBuf::with_secure(32);
//     pw_hash(&mut passphrase,salt,&mut secret);
//     dec(&mut decrypted_message,&mut secret,None,&mut nonce,&mut cipher).unwrap();
// }

pub fn encode_id(sign_pub: &mut SecBuf,enc_pub:&mut SecBuf,id:&mut SecBuf){
    let sign_pub = sign_pub.read_lock();
    let enc_pub = enc_pub.read_lock();
    let mut id = id.write_lock();

    if id.len() == sign_pub.len() + enc_pub.len() {
        for x in 0..sign_pub.len() {
            id[x]=sign_pub[x];
        }
        for x in 0..enc_pub.len() {
            id[x+sign_pub.len()] = enc_pub[x];
        }
    }
    else{
        panic!("The Size of the id secbuf should be : {}",sign_pub.len() + enc_pub.len());
    }
}

pub fn decode_id(id:&mut SecBuf,sign_pub: &mut SecBuf,enc_pub:&mut SecBuf){
    let mut sign_pub = sign_pub.write_lock();
    let mut enc_pub = enc_pub.write_lock();
    let id = id.read_lock();

    if id.len()%2 == 0 {
        for x in 0..sign_pub.len() {
            sign_pub[x]=id[x];
        }
        for x in 0..enc_pub.len() {
            enc_pub[x] = id[x+sign_pub.len()];
        }
    }
    else{
        panic!("The Size of the sign_pub and enc_pub secbuf should be : {}",id.len()/2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holochain_sodium::random::random_secbuf;

    #[test]
    fn it_should_encode_to_create_pub_key() {
        let mut sign_pub = SecBuf::with_insecure(32);
        random_secbuf(&mut sign_pub);

        let mut enc_pub = SecBuf::with_insecure(32);
        random_secbuf(&mut enc_pub);

        let mut id = SecBuf::with_secure(64);


        encode_id(&mut sign_pub,&mut enc_pub,&mut id);
        // assert!(false);
        assert_eq!(sign_pub.len()*2,id.len());
        // assert_eq!(format!("{:?}", *sign_pub),format!("{:?}", *id));
    }

    #[test]
    fn it_should_decode_to_create_pub_key() {
        let mut sign_pub = SecBuf::with_insecure(32);
        let mut enc_pub = SecBuf::with_insecure(32);

        let mut id = SecBuf::with_secure(64);
        random_secbuf(&mut id);

        decode_id(&mut id,&mut sign_pub,&mut enc_pub);
        assert_eq!(sign_pub.len()*2,id.len());
        // assert_eq!(format!("{:?}", *sign_pub),format!("{:?}", *id));
    }
}
