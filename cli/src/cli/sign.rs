use crate::cli::keygen::get_root_seed;
use holochain_core_types::error::HcResult;
use lib3h_sodium::secbuf::SecBuf;


pub fn sign(root_seed_string: Option<String>, message: String) -> HcResult<String> {
    let mut root_seed = get_root_seed(root_seed_string, &String::from(""), false)?;
    let mut revocation_keypair = root_seed.generate_revocation_key()?;
    let mut data_buf = SecBuf::with_insecure_from_string(message);
    let mut signature_buf = revocation_keypair.sign(&mut data_buf)?;
    let buf = signature_buf.read_lock();
    let signature_str = base64::encode(&**buf);
    Ok(signature_str)
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use cli::dpki_init;

    #[test]
    fn can_sign_a_message() {
    	let mnemonic = dpki_init(Some("dummy passphrase".to_string()))
            .expect("Could not generate root seed mneomonic");

        let message = String::from("sign me");
        let signed_message = sign(Some(mnemonic), message);

        assert!(
        	signed_message.is_ok(),
        )
    }
}
