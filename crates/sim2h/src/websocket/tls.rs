extern crate native_tls;
extern crate openssl;

use crate::{websocket::{FAKE_PASS, FAKE_PKCS12},NEW_RELIC_LICENSE_KEY};

use lib3h::transport::error::TransportResult;

use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{self, X509Name, X509},
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

// Generates a key/cert pair
fn generate_pair() -> (PKey<Private>, x509::X509) {
    let rsa = Rsa::generate(2048).unwrap();
    let key = PKey::from_rsa(rsa).unwrap();

    let mut name = X509Name::builder().unwrap();
    name.append_entry_by_nid(openssl::nid::Nid::COMMONNAME, "example.com")
        .unwrap();
    let name = name.build();

    let serial_number = {
        let mut serial = BigNum::new().unwrap();
        serial.rand(159, MsbOption::MAYBE_ZERO, false).unwrap();
        serial.to_asn1_integer().unwrap()
    };

    let mut builder = X509::builder().unwrap();
    builder.set_serial_number(&serial_number).unwrap();
    builder.set_version(2).unwrap();
    builder.set_subject_name(&name).unwrap();
    builder.set_issuer_name(&name).unwrap();
    builder.set_pubkey(&key).unwrap();
    let not_before = Asn1Time::days_from_now(0).unwrap();
    builder.set_not_before(&not_before).unwrap();
    let not_after = Asn1Time::days_from_now(3650).unwrap();
    builder.set_not_after(&not_after).unwrap();
    builder.sign(&key, MessageDigest::sha256()).unwrap();

    let cert: openssl::x509::X509 = builder.build();

    (key, cert)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TlsCertificate {
    pub(in crate::websocket) pkcs12_data: Vec<u8>,
    pub(in crate::websocket) passphrase: String,
}

#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl TlsCertificate {
    /// Creates a self-signed certificate with an entropy key and passphrase.
    /// This makes it possible to use a TLS encrypted connection securely between two
    /// peers using the lib3h websockt actor.
    pub fn build_from_entropy() -> Self {
        let (key, cert) = generate_pair();

        let random_passphrase: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();

        let pkcs12 = openssl::pkcs12::Pkcs12::builder()
            .build(&random_passphrase, "friendly_name", &*key, &cert)
            .unwrap();

        // The DER-encoded bytes of the archive
        let der = pkcs12.to_der().unwrap();

        Self {
            pkcs12_data: der,
            passphrase: random_passphrase,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TlsConfig {
    Unencrypted,
    FakeServer,
    SuppliedCertificate(TlsCertificate),
}

#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl TlsConfig {
    pub fn build_from_entropy() -> Self {
        TlsConfig::SuppliedCertificate(TlsCertificate::build_from_entropy())
    }

    pub fn get_identity(&self) -> TransportResult<native_tls::Identity> {
        Ok(match self {
            TlsConfig::Unencrypted => unimplemented!(),
            TlsConfig::FakeServer => native_tls::Identity::from_pkcs12(FAKE_PKCS12, FAKE_PASS)?,
            TlsConfig::SuppliedCertificate(cert) => {
                native_tls::Identity::from_pkcs12(&cert.pkcs12_data, &cert.passphrase)?
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::websocket::{
        mem_stream::*,
        streams::{StreamEvent, StreamManager},
    };
    use std::io::{Read, Write};
    use url2::prelude::*;

    #[derive(Debug)]
    struct MockStream {
        name: String,
        recv_bytes: Vec<u8>,
        send_bytes: Vec<u8>,
        should_end: bool,
    }

    impl MockStream {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                recv_bytes: Vec::new(),
                send_bytes: Vec::new(),
                should_end: false,
            }
        }

        pub fn inject_recv(&mut self, bytes: Vec<u8>) {
            self.recv_bytes.extend(bytes);
        }

        pub fn drain_send(&mut self) -> Vec<u8> {
            self.send_bytes.drain(..).collect()
        }

        pub fn set_should_end(&mut self) {
            self.should_end = true;
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            println!("{} got read", self.name);

            if self.recv_bytes.len() == 0 {
                if self.should_end {
                    return Ok(0);
                } else {
                    return Err(std::io::ErrorKind::WouldBlock.into());
                }
            }

            let v: Vec<u8> = self
                .recv_bytes
                .drain(0..std::cmp::min(buf.len(), self.recv_bytes.len()))
                .collect();
            buf[0..v.len()].copy_from_slice(&v);
            Ok(v.len())
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            println!("{} got write {}", self.name, buf.len());
            self.send_bytes.extend(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn it_meta_mock_stream_should_work() {
        let mut s = MockStream::new("test");
        s.write_all(b"test").unwrap();
        assert_eq!("test", &String::from_utf8_lossy(&s.drain_send()[..]));
        s.inject_recv(b"hello".to_vec());
        s.set_should_end();
        let mut v = Vec::new();
        s.read_to_end(&mut v).unwrap();
        assert_eq!("hello", &String::from_utf8_lossy(&v[..]));
    }

    enum MockTlsStream {
        Mid(native_tls::MidHandshakeTlsStream<MockStream>),
        Ready(native_tls::TlsStream<MockStream>),
    }

    struct MockConnection {
        srv: Option<MockTlsStream>,
        cli: Option<MockTlsStream>,
        srv_send: Vec<u8>,
        cli_send: Vec<u8>,
        srv_recv: Vec<u8>,
        cli_recv: Vec<u8>,
    }

    impl MockConnection {
        pub fn new(tls_config: TlsConfig) -> Self {
            let connector = native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()
                .unwrap();
            let client_side = match connector.connect("test.test", MockStream::new("client")) {
                Err(native_tls::HandshakeError::WouldBlock(socket)) => socket,
                _ => panic!("unexpected"),
            };

            let identity = tls_config.get_identity().unwrap();
            let acceptor = native_tls::TlsAcceptor::new(identity).unwrap();
            let server_side = match acceptor.accept(MockStream::new("server")) {
                Err(native_tls::HandshakeError::WouldBlock(socket)) => socket,
                _ => panic!("unexpected"),
            };

            let mut out = Self {
                srv: Some(MockTlsStream::Mid(server_side)),
                cli: Some(MockTlsStream::Mid(client_side)),
                srv_send: Vec::new(),
                cli_send: Vec::new(),
                srv_recv: Vec::new(),
                cli_recv: Vec::new(),
            };
            out.process();
            out
        }

        pub fn flush(&mut self) {
            self.srv_send.clear();
            self.cli_send.clear();
            self.srv_recv.clear();
            self.cli_recv.clear();
        }

        pub fn process(&mut self) {
            for _ in 0..10 {
                self.priv_process();
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }

        pub fn write_all(&mut self, is_srv: bool, mut data: &[u8]) {
            for _ in 0..10 {
                match if is_srv { &mut self.srv } else { &mut self.cli } {
                    Some(MockTlsStream::Ready(stream)) => {
                        println!("(is_srv: {}) TRY WRITE {} bytes", is_srv, data.len());
                        match stream.write(data) {
                            Ok(0) => panic!("failed to write"),
                            Ok(n) => data = &data[n..],
                            Err(ref e)
                                if e.kind() == std::io::ErrorKind::Interrupted
                                    || e.kind() == std::io::ErrorKind::WouldBlock =>
                            {
                                println!("-- {:?}", e)
                            }
                            Err(e) => panic!("{:?}", e),
                        }
                        if data.is_empty() {
                            return;
                        }
                    }
                    _ => panic!("unexpected"),
                }
            }
            panic!("failed to write");
        }

        pub fn srv_write(&mut self, data: &[u8]) {
            self.write_all(true, data);
        }

        pub fn cli_write(&mut self, data: &[u8]) {
            self.write_all(false, data);
        }

        fn priv_process_stream(&mut self, stream: MockTlsStream) -> (MockTlsStream, Vec<u8>) {
            let mut data = Vec::new();
            let stream = match stream {
                MockTlsStream::Mid(stream) => match stream.handshake() {
                    Err(native_tls::HandshakeError::WouldBlock(stream)) => {
                        MockTlsStream::Mid(stream)
                    }
                    Ok(stream) => MockTlsStream::Ready(stream),
                    _ => panic!("unexpected"),
                },
                MockTlsStream::Ready(mut stream) => {
                    let mut buf: [u8; 1024] = [0; 1024];
                    match stream.read(&mut buf) {
                        Ok(read) => {
                            if read > 0 {
                                data.extend_from_slice(&buf[0..read]);
                            }
                        }
                        _ => (),
                    }
                    MockTlsStream::Ready(stream)
                }
            };
            (stream, data)
        }

        fn priv_process(&mut self) {
            {
                let srv = match self.srv.as_mut().unwrap() {
                    MockTlsStream::Mid(srv) => srv.get_mut(),
                    MockTlsStream::Ready(srv) => srv.get_mut(),
                };
                let cli = match self.cli.as_mut().unwrap() {
                    MockTlsStream::Mid(cli) => cli.get_mut(),
                    MockTlsStream::Ready(cli) => cli.get_mut(),
                };
                let data = srv.drain_send();
                if data.len() > 0 {
                    self.srv_send.extend_from_slice(&data);
                    cli.inject_recv(data);
                }
                let data = cli.drain_send();
                if data.len() > 0 {
                    self.cli_send.extend_from_slice(&data);
                    srv.inject_recv(data);
                }
            }
            {
                let srv = std::mem::replace(&mut self.srv, None).unwrap();
                let (srv, data) = self.priv_process_stream(srv);
                self.srv_recv.extend_from_slice(&data);
                std::mem::replace(&mut self.srv, Some(srv));
            }
            {
                let cli = std::mem::replace(&mut self.cli, None).unwrap();
                let (cli, data) = self.priv_process_stream(cli);
                self.cli_recv.extend_from_slice(&data);
                std::mem::replace(&mut self.cli, Some(cli));
            }
        }
    }

    fn test_enc_dec(tls_config: TlsConfig) {
        let mut con = MockConnection::new(tls_config);
        con.flush();

        const TO_SERVER: &'static [u8] = b"test-message-to-server";
        con.cli_write(TO_SERVER);

        con.process();

        println!(
            "{:?} -- {:?}",
            String::from_utf8_lossy(&con.srv_recv),
            String::from_utf8_lossy(&con.cli_send),
        );

        assert_ne!(TO_SERVER, &con.cli_send[..]);
        assert_eq!(TO_SERVER, &con.srv_recv[..]);

        con.flush();

        const TO_CLIENT: &'static [u8] = b"test-message-to-client";
        con.srv_write(TO_CLIENT);

        con.process();

        println!(
            "{:?} -- {:?}",
            String::from_utf8_lossy(&con.cli_recv),
            String::from_utf8_lossy(&con.srv_send),
        );

        assert_ne!(TO_CLIENT, &con.srv_send[..]);
        assert_eq!(TO_CLIENT, &con.cli_recv[..]);
    }

    #[test]
    fn it_can_use_fake_server_tls() {
        test_enc_dec(TlsConfig::FakeServer);
    }

    #[test]
    fn it_can_use_self_signed_ephemeral_tls() {
        test_enc_dec(TlsConfig::build_from_entropy());
    }

    use std::collections::HashMap;

    struct StreamTester {
        tls_config: TlsConfig,
        managers: HashMap<Url2, StreamManager<MemStream>>,
    }

    impl StreamTester {
        fn new(tls_config: TlsConfig) -> Self {
            Self {
                tls_config,
                managers: HashMap::new(),
            }
        }

        fn process(&mut self) -> Vec<StreamEvent> {
            let mut out = Vec::new();

            for _ in 0..10 {
                for (_url, manager) in self.managers.iter_mut() {
                    let (_, mut evs) = manager.process().unwrap();
                    out.append(&mut evs);
                }
            }

            out
        }

        fn bind(&mut self, url: Url2) -> Url2 {
            let mut new_manager = StreamManager::with_mem_stream(self.tls_config.clone());
            let url: Url2 = new_manager.bind(&url.into()).unwrap().into();
            self.managers.insert(url.clone(), new_manager);
            url
        }

        fn connect(&mut self, from_url: &Url2, to_url: &Url2) -> Url2 {
            self.managers
                .get_mut(from_url)
                .unwrap()
                .connect(&to_url)
                .unwrap();
            let mut got_in = None;
            let mut got_out = false;
            for ev in self.process() {
                match ev {
                    StreamEvent::IncomingConnectionEstablished(url) => {
                        got_in = Some(url);
                    }
                    StreamEvent::ConnectResult(_url, _id) => {
                        got_out = true;
                    }
                    e @ _ => panic!("unexpected {:?}", e),
                }
            }
            if got_in.is_none() || !got_out {
                panic!("could not connect");
            }
            got_in.unwrap().into()
        }

        fn send(&mut self, from_url: &Url2, to_url: &Url2, data: &[u8]) {
            self.managers
                .get_mut(from_url)
                .unwrap()
                .send(to_url, data)
                .unwrap();
            let mut got = false;
            for ev in self.process() {
                match ev {
                    StreamEvent::ReceivedData(_url, rdata) => {
                        assert_eq!(
                            String::from_utf8_lossy(data),
                            String::from_utf8_lossy(&rdata),
                        );
                        got = true
                    }
                    e @ _ => panic!("unexpected {:?}", e),
                }
            }
            if !got {
                panic!("could not send");
            }
        }

        #[allow(dead_code)]
        fn close(&mut self, from_url: &Url2, to_url: &Url2) {
            self.managers
                .get_mut(from_url)
                .unwrap()
                .close(to_url)
                .unwrap();
        }
    }

    #[test]
    fn it_should_work_with_mem_stream() {
        let mut t = StreamTester::new(TlsConfig::FakeServer);
        let url1 = t.bind(Url2::parse("mem://test1"));
        let url2 = t.bind(Url2::parse("mem://test2"));
        let url_a = t.connect(&url1, &url2);
        t.send(&url1, &url2, b"hello");
        t.send(&url2, &url_a, b"hello2");
    }
}
