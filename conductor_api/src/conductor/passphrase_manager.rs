use crossbeam_channel::{unbounded, Sender};
use holochain_core_types::error::error::HolochainError;
use holochain_sodium::secbuf::SecBuf;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

/// We are caching the passphrase for 10 minutes.
const PASSPHRASE_CACHE_DURATION_SECS: u64 = 600;

pub trait PassphraseService {
    fn request_passphrase(&self) -> Result<SecBuf, HolochainError>;
}

#[derive(Clone)]
pub struct PassphraseManager {
    passphrase_cache: Arc<Mutex<Option<SecBuf>>>,
    passphrase_service: Arc<Mutex<PassphraseService + Send>>,
    last_read: Arc<Mutex<Instant>>,
    timeout_kill_switch: Sender<()>,
}

impl PassphraseManager {
    pub fn new(passphrase_service: Arc<Mutex<PassphraseService + Send>>) -> Self {
        let (kill_switch_tx, kill_switch_rx) = unbounded::<()>();
        let pm = PassphraseManager {
            passphrase_cache: Arc::new(Mutex::new(None)),
            passphrase_service,
            last_read: Arc::new(Mutex::new(Instant::now())),
            timeout_kill_switch: kill_switch_tx,
        };

        let pm_clone = pm.clone();

        thread::spawn(move || loop {
            if kill_switch_rx.try_recv().is_ok() {
                return;
            }

            if pm_clone.passphrase_cache.lock().unwrap().is_some() {
                let duration_since_last_read =
                    Instant::now().duration_since(*pm_clone.last_read.lock().unwrap());

                if duration_since_last_read > Duration::from_secs(PASSPHRASE_CACHE_DURATION_SECS) {
                    pm_clone.forget_passphrase();
                }
            }

            thread::sleep(Duration::from_secs(1));
        });

        pm
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

        *(self.last_read.lock().unwrap()) = Instant::now();

        match *passphrase {
            Some(ref mut passphrase_buf) => {
                let mut new_passphrase_buf = SecBuf::with_insecure(passphrase_buf.len());
                new_passphrase_buf.write(0, &*(passphrase_buf.read_lock()))?;
                Ok(new_passphrase_buf)
            }
            None => unreachable!(),
        }
    }

    fn forget_passphrase(&self) {
        let mut passphrase = self.passphrase_cache.lock().unwrap();
        *passphrase = None;
    }
}

impl Drop for PassphraseManager {
    fn drop(&mut self) {
        let _ = self.timeout_kill_switch.send(());
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

pub struct PassphraseServiceMock {
    pub passphrase: String,
}

impl PassphraseService for PassphraseServiceMock {
    fn request_passphrase(&self) -> Result<SecBuf, HolochainError> {
        Ok(SecBuf::with_insecure_from_string(self.passphrase.clone()))
    }
}
