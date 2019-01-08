use holochain_sodium::{
    secbuf::{SecBuf},
    pwhash,
    aead,
    kx,
};

// allow overrides for unit-testing purposes
pub const PW_HASH_OPS_LIMIT :u64 = pwhash::OPSLIMIT_SENSITIVE;
pub const PW_HASH_MEM_LIMIT :usize = pwhash::MEMLIMIT_SENSITIVE;
pub const PW_HASH_ALGO :i8 = pwhash::ALG_ARGON2ID13;

pub struct Bundle {
    pub bundle_type:String, 
    pub hint:String,
    pub pw_pub_keys: ReturnBundleData, 
    pub pw_sign_priv: ReturnBundleData,
    pub pw_enc_priv: ReturnBundleData,
}
/**
 * simplify the api for generating a password hash with our set parameters
 * @param {SecBuf} pass - the password buffer to hash
 * @param {Buffer} [salt] - if specified, hash with this salt (otherwise random)
 * @return {object} - { salt: Buffer, hash: SecBuf }
 */
pub fn pw_hash(password: &mut SecBuf,salt:&mut SecBuf,hash:&mut SecBuf){
    let mut password = password;
    let mut salt = salt;
    let mut hash = hash;
    pwhash::hash(&mut password,PW_HASH_OPS_LIMIT,PW_HASH_MEM_LIMIT,PW_HASH_ALGO,&mut salt,&mut hash).unwrap()
}

pub struct ReturnBundleData  {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub cipher: Vec<u8>,
}

/**
 * Helper for encrypting a buffer with a pwhash-ed passphrase
 * @param {Buffer} data
 * @param {string} passphrase
 * @return {Buffer} - the encrypted data
 */
pub fn pw_enc(data:&mut SecBuf,passphrase:&mut SecBuf)->ReturnBundleData{
    let mut secret = SecBuf::with_secure(kx::SESSIONKEYBYTES);
    let mut salt = SecBuf::with_secure(pwhash::SALTBYTES);
    holochain_sodium::random::random_secbuf(&mut salt);
    let mut nonce = SecBuf::with_insecure(aead::NONCEBYTES);
    holochain_sodium::random::random_secbuf(&mut nonce);
    let mut cipher = SecBuf::with_insecure(data.len() + aead::ABYTES);
    let mut passphrase = passphrase;
    let mut data = data;
    pw_hash(&mut passphrase,&mut salt,&mut secret);
    aead::enc(&mut data,&mut secret,None,&mut nonce,&mut cipher).unwrap();
   
    let salt = salt.read_lock();
    let nonce = nonce.read_lock();
    let cipher = cipher.read_lock();
    let salt = &*salt;
    let nonce = &*nonce;
    let cipher = &*cipher;
    let salt: Vec<u8> = salt.iter().cloned().collect();
    let nonce: Vec<u8> = nonce.iter().cloned().collect();
    let cipher: Vec<u8> = cipher.iter().cloned().collect();
    let data = ReturnBundleData {
        salt,
        nonce,
        cipher,
    };
    data
}

/**
 * Helper for decrypting a buffer with a pwhash-ed passphrase
 * @param {Buffer} data
 * @param {string} passphrase
 * @return {Buffer} - the decrypted data
 */
pub fn pw_dec (bundle:&ReturnBundleData,passphrase:&mut SecBuf)->SecBuf{
    let mut secret = SecBuf::with_secure(kx::SESSIONKEYBYTES);
    let mut salt = SecBuf::with_secure(pwhash::SALTBYTES);    
    load_secbuf(&bundle.salt,&mut salt);
    let mut nonce = SecBuf::with_insecure(bundle.nonce.len());    
    load_secbuf(&bundle.nonce,&mut nonce);
    let mut cipher = SecBuf::with_insecure(bundle.cipher.len());    
    load_secbuf(&bundle.cipher,&mut cipher);
    let mut passphrase = passphrase;
    pw_hash(&mut passphrase,&mut salt,&mut secret);
    let mut decrypted_message = SecBuf::with_insecure(cipher.len() - aead::ABYTES);
    println!("{} {} {} {} {}",secret.len(),salt.len(),nonce.len(),cipher.len(),decrypted_message.len());
    aead::dec(&mut decrypted_message,&mut secret,None,&mut nonce,&mut cipher).unwrap();
    decrypted_message
}

pub fn load_secbuf(data:&Vec<u8>,buf:&mut SecBuf){
    let mut buf = buf.write_lock();
    for x in 0..data.len() {
        buf[x]=data[x];
        print!("({}):{:?}",x,buf[x]);
    }
    println!("");
}

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


pub fn check_if_wrong_secbuf(buf:&mut SecBuf)->bool{
    let buf = buf.read_lock();
    println!("Buf{:?}",*buf);
    for i in 0..buf.len(){
        if buf[i]!=0{
            return true;
        }
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holochain_sodium::random::random_secbuf;

    #[test]
    fn it_should_encrypt_data() {
        let mut data = SecBuf::with_insecure(32);
        {
            let mut data = data.write_lock();
            data[0] = 88;
            data[1] = 101;
        }
        let mut password = SecBuf::with_secure(pwhash::HASHBYTES);
        {
            let mut password = password.write_lock();
            password[0] = 42;
            password[1] = 222;
        }
        let mut bundle:ReturnBundleData = pw_enc(&mut data,&mut password);

        let mut dec_mess = pw_dec(&mut bundle,&mut password);

        let data = data.read_lock();
        let dec_mess = dec_mess.read_lock();
        assert_eq!(format!("{:?}", *dec_mess),format!("{:?}", *data));  
    }

    #[test]
    fn it_should_generate_pw_hash_with_salt() {
        let mut password = SecBuf::with_secure(pwhash::HASHBYTES);
        let mut pw2_hash = SecBuf::with_secure(pwhash::HASHBYTES);
        {
            let mut password = password.write_lock();
            password[0] = 42;
            password[1] = 222;
        }
        let mut salt = SecBuf::with_insecure(pwhash::SALTBYTES);
        pw_hash(
            &mut password,
            &mut salt,
            &mut pw2_hash,
        );
        let pw2_hash = pw2_hash.read_lock();
        assert_eq!("[84, 166, 168, 46, 130, 222, 122, 144, 123, 49, 206, 167, 35, 180, 246, 154, 25, 43, 218, 177, 95, 218, 12, 241, 234, 207, 230, 93, 127, 174, 221, 106]",  format!("{:?}", *pw2_hash));
    }

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
