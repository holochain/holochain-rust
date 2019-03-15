use holochain_core_types::error::error::HolochainError;
use holochain_sodium::secbuf::SecBuf;
use std::sync::{Arc, Mutex};

pub trait PassphraseService {
    fn request_passphrase(&self) -> Result<SecBuf, HolochainError>;
}

pub struct PassphraseManager {
    passphrase_cache: Arc<Mutex<Option<SecBuf>>>,
    passphrase_service: Arc<Mutex<PassphraseService>>,
}

unsafe impl Send for PassphraseManager {}

impl PassphraseManager {
    pub fn new(passphrase_service: Arc<Mutex<PassphraseService>>) -> Self {
        PassphraseManager {
            passphrase_cache: Arc::new(Mutex::new(None)),
            passphrase_service,
        }
    }

    pub fn get_passphrase(&self) -> Result<SecBuf, HolochainError> {
        let mut passphrase = self.passphrase_cache.lock().unwrap();
        if passphrase.is_none() {
            *passphrase = Some(
                self.passphrase_service
                    .lock()
                    .unwrap()
                    .request_passphrase()?,
            );
        }

        match *passphrase {
            Some(ref mut passphrase_buf) => {
                let mut new_passphrase_buf = SecBuf::with_insecure(passphrase_buf.len());
                new_passphrase_buf.write(0, &*(passphrase_buf.read_lock()))?;
                Ok(new_passphrase_buf)
            }
            None => unreachable!(),
        }
    }
}

pub struct PassphraseServiceCmd {}
impl PassphraseService for PassphraseServiceCmd {
    fn request_passphrase(&self) -> Result<SecBuf, HolochainError> {
        // Prompt for passphrase
        let mut passphrase_string = rpassword::read_password_from_tty(Some("Passphrase: "))?;

        // Move passphrase in secure memory
        let passphrase_bytes = unsafe { passphrase_string.as_mut_vec() };
        let mut passphrase_buf = SecBuf::with_insecure(passphrase_bytes.len());
        passphrase_buf.write(0, passphrase_bytes.as_slice())?;

        // Overwrite the unsafe passphrase memory with zeros
        for byte in passphrase_bytes.iter_mut() {
            *byte = 0u8;
        }

        Ok(passphrase_buf)
    }
}
