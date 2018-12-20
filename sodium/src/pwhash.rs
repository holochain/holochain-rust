//! This module provides access to libsodium

use super::secbuf::SecBuf;
use super::random::buf;

pub const OPSLIMIT_INTERACTIVE:u64 = rust_sodium_sys::crypto_pwhash_OPSLIMIT_INTERACTIVE as u64;
pub const MEMLIMIT_INTERACTIVE:usize = rust_sodium_sys::crypto_pwhash_MEMLIMIT_INTERACTIVE as usize;
pub const OPSLIMIT_MODERATE:u64 = rust_sodium_sys::crypto_pwhash_OPSLIMIT_MODERATE as u64;
pub const MEMLIMIT_MODERATE:usize = rust_sodium_sys::crypto_pwhash_MEMLIMIT_MODERATE as usize;
pub const OPSLIMIT_SENSITIVE:u64 = rust_sodium_sys::crypto_pwhash_OPSLIMIT_SENSITIVE as u64;
pub const MEMLIMIT_SENSITIVE:usize = rust_sodium_sys::crypto_pwhash_MEMLIMIT_SENSITIVE as usize;

pub const ALG_ARGON2I13:i8 = rust_sodium_sys::crypto_pwhash_ALG_ARGON2I13 as i8;
pub const ALG_ARGON2ID13:i8 = rust_sodium_sys::crypto_pwhash_ALG_ARGON2ID13 as i8;

pub const HASHBYTES:usize = 32 as usize;
pub const SALTBYTES:usize = rust_sodium_sys::crypto_pwhash_SALTBYTES as usize;

// fn _fixOpts (opts) {
//
// }

/// Calculate a password hash
/// @example
/// const { salt, hash } = mosodium.pwhash.hash(passphrase)
/// @example
/// const { salt, hash } = mosodium.pwhash.hash(passphrase, {
///   opslimit: mosodium.pwhash.OPSLIMIT_MODERATE,
///   memlimit: mosodium.pwhash.MEMLIMIT_MODERATE,
///   salt: mysalt
/// })
/// @param {SecBuf} password - the password to hash
/// @param {object} opts
/// @param {number} opts.opslimit - operation scaling for hashing algorithm
/// @param {number} opts.memlimit - memory scaling for hashing algorithm
/// @param {number} opts.algorithm - which hashing algorithm
/// @param {Buffer} [opts.salt] - predefined salt (random if not included)
/// @return {object} - { salt / the salt used /, hash / the hash generated / }

pub fn hash(password: &mut SecBuf,ops_limit:u64,mem_limit:usize,alg:i8,salt:Option<&mut SecBuf>,hash:&mut SecBuf){
    // TODO: fix opts
    // let mut hash = SecBuf::with_secure(HASHBYTES);

    match salt {
        Some(salt) => {
            let mut password = password.write_lock();
            let mut salt = salt.write_lock();
            let mut hash = hash.write_lock();
            create_hash(&mut password,ops_limit,mem_limit,alg,&mut salt,&mut hash);
        },
        None => {
            let mut salt = SecBuf::with_insecure(SALTBYTES);
            buf(&mut salt);
            let mut password = password.write_lock();
            let mut salt = salt.write_lock();
            let mut hash = hash.write_lock();
            create_hash(&mut password,ops_limit,mem_limit,alg,&mut salt,&mut hash);


        },
    };
    // return (hash);
}

pub fn create_hash(password: &mut SecBuf,ops_limit:u64,mem_limit:usize,alg:i8,salt:&mut SecBuf,hash :&mut SecBuf){
    println!("See salt: {:?}",salt);
    unsafe{
        let mut password = password.read_lock();
        let mut hash = hash.write_lock();
        let hash_len = hash.len() as libc::c_ulonglong;
        let pw_len = password.len() as libc::c_ulonglong;
        rust_sodium_sys::crypto_pwhash(raw_ptr_char!(hash),hash_len,raw_ptr_ichar_immut!(password),pw_len,raw_ptr_char_immut!(salt),ops_limit as libc::c_ulonglong,mem_limit,alg as libc::c_int);
        println!("> hash : {:?}",hash);

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_generate_with_no_salt() {
        let mut password = SecBuf::with_secure(HASHBYTES);
        let mut pw1_hash = SecBuf::with_secure(HASHBYTES);
        buf(&mut password);
        hash(&mut password,OPSLIMIT_SENSITIVE,MEMLIMIT_SENSITIVE,ALG_ARGON2ID13,None,&mut pw1_hash);
        let mut pw1_hash = pw1_hash.write_lock();
        println!("pw 1 : {:?}",pw1_hash);
        let mut password = password.write_lock();
        assert_eq!(HASHBYTES, password.len());
    }
    #[test]
    fn it_should_generate_with_salt() {
        let mut password = SecBuf::with_secure(HASHBYTES);
        let mut pw2_hash = SecBuf::with_secure(HASHBYTES);
        buf(&mut password);
        let mut salt = SecBuf::with_insecure(SALTBYTES);
        hash(&mut password,OPSLIMIT_SENSITIVE,MEMLIMIT_SENSITIVE,ALG_ARGON2ID13,Some(&mut salt),&mut pw2_hash);
        let mut pw2_hash = pw2_hash.write_lock();
        println!("pw 2 : {:?}",pw2_hash);
        let mut password = password.write_lock();
        assert_eq!(HASHBYTES, password.len());
    }
    #[test]
    fn it_should_generate_consistantly() {
        let mut password = SecBuf::with_secure(HASHBYTES);
        let mut pw1_hash = SecBuf::with_secure(HASHBYTES);
        let mut pw2_hash = SecBuf::with_secure(HASHBYTES);
        buf(&mut password);
        let mut salt = SecBuf::with_insecure(SALTBYTES);
        hash(&mut password,OPSLIMIT_SENSITIVE,MEMLIMIT_SENSITIVE,ALG_ARGON2ID13,Some(&mut salt),&mut pw1_hash);
        hash(&mut password,OPSLIMIT_SENSITIVE,MEMLIMIT_SENSITIVE,ALG_ARGON2ID13,Some(&mut salt),&mut pw2_hash);
        let mut pw1_hash = pw1_hash.write_lock();
        let mut pw2_hash = pw2_hash.write_lock();
        let mut password = password.write_lock();
        println!("password : {:?}",password);
        println!("pw 1 : {:?}",pw1_hash);
        println!("pw 2 : {:?}",pw2_hash);
        let mut password = password.write_lock();
        assert_eq!(HASHBYTES, password.len());
    }

}
