use std::sync::Arc;

use crate::clients::config::Config;
use anyhow::Result;
use std::{fs::File, io::BufReader};
use tokio_rustls::rustls::Certificate;
use tokio_rustls::{rustls, TlsConnector};

pub fn connector(config: Arc<Config>) -> Result<TlsConnector> {
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
        // .with_client_auth_cert()
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));

    Ok(connector)
}
