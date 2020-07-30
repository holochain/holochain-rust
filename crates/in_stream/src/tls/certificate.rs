static FAKE_PKCS12: &[u8] = include_bytes!("fake_key.p12");
static FAKE_PASS: &str = "hello";

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
        let id = format!("a{}a.a{}a", nanoid::simple(), nanoid::simple());
        let mut params = rcgen::CertificateParams::new(vec![id.into()]);
        // would be nice to ed25519 - but seems incompatible with this openssl
        //params.alg = &rcgen::PKCS_ED25519;
        params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
        let cert = rcgen::Certificate::from_params(params).expect("gen cert");

        let key = cert.serialize_private_key_der();
        let key = openssl::pkey::PKey::private_key_from_der(&key).expect("private key");

        let cert = cert.serialize_der().expect("cert der");
        let cert = openssl::x509::X509::from_der(&cert).expect("cert der");

        let pkcs12 = openssl::pkcs12::Pkcs12::builder()
            .build("dev-passphrase", "in_stream_tls", &key, &cert)
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
