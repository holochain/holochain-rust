//! This module provides access to libsodium

use super::secbuf::SecBuf;
use super::random::buf;

pub const PUBLICKEYBYTES:usize = rust_sodium_sys::crypto_kx_PUBLICKEYBYTES as usize;
pub const SECRETKEYBYTES:usize = rust_sodium_sys::crypto_kx_SECRETKEYBYTES as usize;
pub const SESSIONKEYBYTES:usize = rust_sodium_sys::crypto_kx_SESSIONKEYBYTES as usize;

/// Generate a fresh, random keyexchange keypair
/// @example
/// const { publicKey, secretKey } = mosodium.kx.keypair()
/// @return {object} { publicKey, secretKey }
pub fn keypair(pk: &mut SecBuf,sk:&mut SecBuf){
    unsafe{
        let mut pk = pk.write_lock();
        let mut sk = sk.write_lock();
        rust_sodium_sys::crypto_kx_keypair(raw_ptr_char!(pk),raw_ptr_char!(sk));
    }
}


/// Generate a fresh, keyexchange keypair, based off a seed
/// @example
/// const { publicKey, secretKey } = mosodium.kx.seedKeypair(seed)
/// @param {SecBuf} seed - the seed to derive a keypair from
/// @return {object} { publicKey, secretKey }
pub fn seed_keypair(seed: &mut SecBuf,pk: &mut SecBuf,sk: &mut SecBuf){
    unsafe{
        let mut seed = seed.read_lock();
        let mut pk = pk.write_lock();
        let mut sk = sk.write_lock();
        rust_sodium_sys::crypto_kx_seed_keypair(raw_ptr_char!(pk),raw_ptr_char!(sk),raw_ptr_char_immut!(seed));
    }
}

/**
 * Given a server's public key, derive shared secrets.
 * @example
 * const { rx, tx } = mosodium.kx.clientSession(cliPub, cliSec, srvPub)
 *
 * @param {Buffer} cliPublic - client's public key
 * @param {SecBuf} cliSecret - client's secret key
 * @param {Buffer} srvPublic - server's public key
 * @return {object} { rx /receive key/, tx /transmit key/ }
 */
pub fn client_session(client_pk: &mut SecBuf,client_sk: &mut SecBuf,server_pk: &mut SecBuf,rx: &mut SecBuf,tx: &mut SecBuf){
    unsafe{
        let mut rx = rx.write_lock();
        let mut tx = tx.write_lock();
        let mut client_sk = client_sk.read_lock();
        let mut client_pk = client_pk.read_lock();
        let mut server_pk = server_pk.read_lock();
        rust_sodium_sys::crypto_kx_client_session_keys(raw_ptr_char!(rx),raw_ptr_char!(tx),raw_ptr_char_immut!(client_pk),raw_ptr_char_immut!(client_sk),raw_ptr_char_immut!(server_pk));
    }
}

/**
 * Given a client's public key, derive shared secrets.
 * @example
 * const { rx, tx } = mosodium.kx.serverSession(srvPub, srvSec, cliPub)
 *
 * @param {Buffer} srvPublic - server's public key
 * @param {SecBuf} srvSecret - server's secret key
 * @param {Buffer} cliPublic - client's public key
 * @return {object} { rx /receive key/, tx /transmit key/ }
 */
 pub fn server_session(server_pk: &mut SecBuf,server_sk: &mut SecBuf,client_pk: &mut SecBuf,rx: &mut SecBuf,tx: &mut SecBuf){
     unsafe{
         let mut rx = rx.write_lock();
         let mut tx = tx.write_lock();
         let mut client_pk = client_pk.read_lock();
         let mut server_sk = server_sk.read_lock();
         let mut server_pk = server_pk.read_lock();
         rust_sodium_sys::crypto_kx_server_session_keys(raw_ptr_char!(rx),raw_ptr_char!(tx),raw_ptr_char_immut!(server_pk),raw_ptr_char_immut!(server_sk),raw_ptr_char_immut!(client_pk));
     }
 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_generate_keypair() {
        let mut public_key = SecBuf::with_secure(PUBLICKEYBYTES);
        let mut secret_key = SecBuf::with_secure(SECRETKEYBYTES);
        keypair(&mut public_key,&mut secret_key);
        let mut public_key = public_key.read_lock();
        let mut secret_key = secret_key.read_lock();
        println!("public_key : {:?}",public_key);
        println!("secret_key : {:?}",secret_key);
        assert_eq!(32, public_key.len());
        assert_eq!(32, secret_key.len());
    }
    #[test]
    fn it_should_generate_keypair_from_seed() {
        let mut seed = SecBuf::with_secure(32);
        buf(&mut seed);
        let mut public_key = SecBuf::with_secure(PUBLICKEYBYTES);
        let mut secret_key = SecBuf::with_secure(SECRETKEYBYTES);
        seed_keypair(&mut seed,&mut public_key,&mut secret_key);
        let mut public_key = public_key.read_lock();
        let mut secret_key = secret_key.read_lock();
        println!("public_key : {:?}",public_key);
        println!("secret_key : {:?}",secret_key);
        assert_eq!(32, public_key.len());
        assert_eq!(32, secret_key.len());
    }
    #[test]
    fn it_should_generate_client_keys() {
        let mut client_pk = SecBuf::with_secure(PUBLICKEYBYTES);
        let mut client_sk = SecBuf::with_secure(SECRETKEYBYTES);
        keypair(&mut client_pk,&mut client_sk);

        let mut server_pk = SecBuf::with_secure(PUBLICKEYBYTES);
        let mut server_sk = SecBuf::with_secure(SECRETKEYBYTES);
        keypair(&mut server_pk,&mut server_sk);

        let mut cli_rx = SecBuf::with_secure(SESSIONKEYBYTES);
        let mut cli_tx = SecBuf::with_secure(SESSIONKEYBYTES);
        {
            client_session(&mut client_pk,&mut client_sk,&mut server_pk,&mut cli_rx,&mut cli_tx);
            let mut cli_rx = cli_rx.read_lock();
            let mut cli_tx = cli_tx.read_lock();
            println!("cli_rx : {:?}",cli_rx);
            println!("cli_tx : {:?}",cli_tx);
        }
        let mut srv_rx = SecBuf::with_secure(SESSIONKEYBYTES);
        let mut srv_tx = SecBuf::with_secure(SESSIONKEYBYTES);
        {
            server_session(&mut server_pk,&mut server_sk,&mut client_pk,&mut srv_rx,&mut srv_tx);
            let mut srv_rx = srv_rx.read_lock();
            let mut srv_tx = srv_tx.read_lock();
            println!("srv_rx : {:?}",srv_rx);
            println!("srv_tx : {:?}",srv_tx);
            assert_eq!(32, srv_tx.len());
        }
    }
}
