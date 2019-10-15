use crossbeam_channel::{unbounded, Sender};
use holochain_core_types::{error::HolochainError, sync::HcMutex as Mutex};
use lib3h_sodium::secbuf::SecBuf;
#[cfg(unix)]
use log::Level;
use std::{
    io::{self, Write},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::io::{BufRead, BufReader};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

/// We are caching the passphrase for 10 minutes.
const PASSPHRASE_CACHE_DURATION_SECS: u64 = 600;

pub trait PassphraseService {
    fn request_passphrase(&self) -> Result<SecBuf, HolochainError>;
}

#[derive(Clone)]
pub struct PassphraseManager {
    passphrase_cache: Arc<Mutex<Option<SecBuf>>>,
    passphrase_service: Arc<Mutex<dyn PassphraseService + Send>>,
    last_read: Arc<Mutex<Instant>>,
    timeout_kill_switch: Sender<()>,
}

impl PassphraseManager {
    pub fn new(passphrase_service: Arc<Mutex<dyn PassphraseService + Send>>) -> Self {
        let (kill_switch_tx, kill_switch_rx) = unbounded::<()>();
        let pm = PassphraseManager {
            passphrase_cache: Arc::new(Mutex::new(None)),
            passphrase_service,
            last_read: Arc::new(Mutex::new(Instant::now())),
            timeout_kill_switch: kill_switch_tx,
        };

        let pm_clone = pm.clone();

        let _ = thread::Builder::new()
            .name("passphrase_manager".to_string())
            .spawn(move || loop {
                if kill_switch_rx.try_recv().is_ok() {
                    return;
                }

                if pm_clone.passphrase_cache.lock().unwrap().is_some() {
                    let duration_since_last_read =
                        Instant::now().duration_since(*pm_clone.last_read.lock().unwrap());

                    if duration_since_last_read
                        > Duration::from_secs(PASSPHRASE_CACHE_DURATION_SECS)
                    {
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
        print!("Passphrase: ");
        io::stdout().flush().expect("Could not flush stdout!");
        let mut passphrase_string = rpassword::read_password()?;

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

#[cfg(unix)]
pub struct PassphraseServiceUnixSocket {
    path: String,
    stream: Arc<Mutex<Option<std::io::Result<BufReader<UnixStream>>>>>,
}

#[cfg(unix)]
impl PassphraseServiceUnixSocket {
    pub fn new(path: String) -> Self {
        let stream = Arc::new(Mutex::new(None));
        let stream_clone = stream.clone();
        let listener = UnixListener::bind(path.clone())
            .expect("Could not create unix socket for passphrase service");
        log_info!("Start accepting passphrase IPC connections on socket...");
        thread::spawn(move || {
            let accept_result = listener.accept();
            {
                *(stream_clone.lock().unwrap()) =
                    Some(accept_result.map(|(stream, _)| BufReader::new(stream)));
            }
            log_info!("Passphrase provider connected through unix socket");
        });
        PassphraseServiceUnixSocket { path, stream }
    }
}

#[cfg(unix)]
impl Drop for PassphraseServiceUnixSocket {
    fn drop(&mut self) {
        std::fs::remove_file(self.path.clone()).unwrap();
    }
}

#[cfg(unix)]
impl PassphraseService for PassphraseServiceUnixSocket {
    fn request_passphrase(&self) -> Result<SecBuf, HolochainError> {
        log_debug!("Passphrase needed. Using unix socket passphrase service...");
        while self.stream.lock().unwrap().is_none() {
            log_debug!("No one connected via socket yet. Waiting...");
            thread::sleep(Duration::from_millis(500));
        }

        log_debug!("We have an open connection to a passphrase provider.");

        // Request and read passphrase from socket
        let mut passphrase_string = {
            let mut stream_option = self.stream.lock().expect(
                "Could not lock mutex holding unix domain socket connection for passphrase service",
            );
            let listen_result = stream_option.inner.as_mut()
                .expect("This option must be some at this point since we would be in above while loop otherwise.");
            let stream = listen_result
                .as_mut()
                .expect("Error accepting unix socket connection for passphrase service");

            log_debug!("Sending passphrase request via unix socket...");
            stream
                .get_mut()
                .write_all(b"request_passphrase")
                .expect("Could not write to passphrase socket");
            log_debug!("Passphrase request sent.");
            let mut passphrase_string = String::new();
            log_debug!("Reading passphrase from socket...");
            stream
                .read_line(&mut passphrase_string)
                .expect("Could not read from passphrase socket");
            log_debug!("Got passphrase. All fine.");
            passphrase_string
        };

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
