static FAKE_PKCS12: &[u8] = include_bytes!("fake_key.p12");
static FAKE_PASS: &str = "hello";

use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{self, X509Name, X509},
};

type PrivateKey = PKey<Private>;
type Certificate = x509::X509;

/// private helper - generate a self-signed cert given an x509 name
fn generate_self_signed(name: X509Name) -> (PrivateKey, Certificate) {
    let rsa = Rsa::generate(2048).unwrap();
    let key = PKey::from_rsa(rsa).unwrap();

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

    let cert: Certificate = builder.build();

    (key, cert)
}

/// private helper - generate a self-signed dev certificate
fn generate_dev() -> (PrivateKey, Certificate) {
    let o = "InStreamDevCertificate";
    let cn = nanoid::simple();

    let mut name = X509Name::builder().unwrap();
    name.append_entry_by_nid(openssl::nid::Nid::ORGANIZATIONNAME, o)
        .unwrap();
    name.append_entry_by_nid(openssl::nid::Nid::COMMONNAME, &cn)
        .unwrap();
    let name = name.build();

    generate_self_signed(name)
}

/// represents an encrypted TLS certificate, and the passphrase to decrypt it
/// obviously, when serializing, you should only encode the data, not the passphrase
#[derive(Debug, Clone, PartialEq)]
pub struct TlsCertificate {
    pub pkcs12_data: Vec<u8>,
    pub passphrase: String,
}

impl TlsCertificate {
    /// generate a self-signed dev certificate
    pub fn generate_dev() -> Self {
        let (key, cert) = generate_dev();

        let pkcs12 = openssl::pkcs12::Pkcs12::builder()
            .build("dev-passphrase", "in_stream_tls", &*key, &cert)
            .unwrap();

        Self {
            pkcs12_data: pkcs12.to_der().unwrap(),
            passphrase: "dev-passphrase".to_string(),
        }
    }

    /// WARNING - do not use this with any sensitive data
    ///         - the private key is PUBLIC
    /// use a pre-generated fake certificate
    /// speeds up unit tests, because we don't have to generate an RSA keypair
    pub fn with_fake_certificate() -> Self {
        Self {
            pkcs12_data: FAKE_PKCS12.to_vec(),
            passphrase: FAKE_PASS.to_string(),
        }
    }
}
