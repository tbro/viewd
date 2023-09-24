use std::io;
use rustls_pemfile::certs;
use std::path::Path;
use std::sync::Arc;

use crate::clients::config::Config;
use anyhow::Result;
use std::{fs::File, io::BufReader};
use tokio_rustls::rustls::{Certificate, PrivateKey};
use tokio_rustls::{rustls, TlsConnector};

pub fn connector(config: Arc<Config>) -> Result<TlsConnector> {
    let certs = load_certs(config.cert.as_path())?;
    let key = load_keys(config.key.as_path())?;

    let mut store = rustls::RootCertStore::empty();

    let f = File::open(config.ca.as_path())?;
    let mut pem = BufReader::new(f);

    for cert in rustls_pemfile::certs(&mut pem)? {
        let cert = Certificate(cert);
        store.add(&cert)?;
    }

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(store)
        .with_client_auth_cert(certs, PrivateKey(key))?;
    let connector = TlsConnector::from(Arc::new(config));

    Ok(connector)
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
