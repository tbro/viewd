use anyhow::{anyhow, Result};
use rustls_pemfile::certs;
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
    sync::Arc,
};
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};

use super::Config;

pub fn get_acceptor(config: Arc<Config>) -> Result<TlsAcceptor> {
    let certs = load_certs(config.cert.as_path())?;
    let key = load_keys(config.key.as_path())?;

    // let mut root_cert_store = rustls::RootCertStore::empty();
    // let client_auth = AllowAnyAuthenticatedClient::new(roots);
    let config = ServerConfig::builder()
        .with_safe_defaults()
        // .with_client_cert_verifier(client_auth)
        .with_no_client_auth()
        .with_single_cert(certs, PrivateKey(key))
        .map_err(|err| anyhow!(err))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    Ok(acceptor)
}

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    let c = certs(&mut BufReader::new(File::open(path)?))?;
    let certs = c.into_iter().map(Certificate).collect();
    Ok(certs)
}

fn load_keys(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let pem = rustls_pemfile::pkcs8_private_keys(&mut reader)?;
    pem.into_iter().next().ok_or(io::ErrorKind::NotFound.into())
}

// here for reference
fn _identify_pem(path: &Path) -> io::Result<()> {
    use std::iter;

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    use rustls_pemfile::{read_one, Item};
    for item in iter::from_fn(|| read_one(&mut reader).transpose()) {
        match item.unwrap() {
            Item::X509Certificate(cert) => println!("certificate {:?}", cert),
            Item::Crl(crl) => println!("certificate revocation list: {:?}", crl),
            Item::RSAKey(key) => println!("rsa pkcs1 key {:?}", key),
            Item::PKCS8Key(key) => println!("pkcs8 key {:?}", key),
            Item::ECKey(key) => println!("sec1 ec key {:?}", key),
            _ => println!("unhandled item"),
        }
    }
    Ok(())
}
